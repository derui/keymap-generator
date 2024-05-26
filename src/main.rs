use std::{
    collections::HashMap,
    env::args,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::SystemTime,
};

use frequency_table::FrequencyTable;
use keymap::Keymap;
use postcard::{from_bytes, to_allocvec};
use rand::{random, rngs::StdRng, SeedableRng};
use score::Conjunction;

use crate::{
    connection_score::{ConnectionScore, TwoKeyTiming},
    playground::Playground,
};

mod char_def;
mod connection_score;
mod frequency_layer;
mod frequency_table;
mod key_def;
mod key_seq;
mod keymap;
mod layout;
mod playground;
mod score;

fn read_4gram(path: &Path) -> anyhow::Result<Vec<Conjunction>> {
    let mut conjunctions = Vec::new();
    let file = File::open(path).unwrap();

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(&file);

    let char_position_map: HashMap<char, (u64, usize)> = char_def::all_chars()
        .into_iter()
        .enumerate()
        .map(|(idx, v)| (v.1, (v.0, idx)))
        .collect();

    for result in rdr.records() {
        // The iterator yields Result<StringRecord, Error>, so we check the
        // error here.
        let record = result?;
        let text: String = record.get(0).unwrap().to_string();
        let appearances: u32 = record.get(1).unwrap().parse()?;

        let filtered = text
            .chars()
            .filter_map(|v| char_position_map.get(&v))
            .cloned()
            .collect::<Vec<_>>();
        if filtered.len() != text.chars().collect::<Vec<_>>().len() {
            continue;
        }

        let hash = filtered
            .iter()
            .map(|(v, _)| v)
            .fold(1, |accum, v| accum * *v);

        conjunctions.push(Conjunction {
            text: filtered.iter().cloned().map(|v| v.1).collect(),
            appearances,
            hash,
        });
    }

    log::info!("log load {} 4-grams as conjunction", conjunctions.len());

    Ok(conjunctions)
}

fn save_frequency(table: &FrequencyTable) {
    let mut output = File::create("./frequency_table.bin").unwrap();
    let bin = to_allocvec(&table).unwrap();
    output.write_all(&bin).unwrap();
}

fn read_frequency(path: &Path) -> anyhow::Result<FrequencyTable> {
    let mut input = File::open(fs::canonicalize(path)?)?;
    let mut buf = Vec::new();
    input.read_to_end(&mut buf)?;
    let data = from_bytes::<FrequencyTable>(&buf)?;
    log::info!("frequency loaded");
    Ok(data)
}

struct Bench {
    last_time: SystemTime,
    generations_count: u64,
}

impl Bench {
    fn new() -> Self {
        Bench {
            last_time: SystemTime::now(),
            generations_count: 0,
        }
    }

    fn update(&mut self, total_generations_count: u64, scores: &[u64]) {
        self.generations_count += 1;
        let now = SystemTime::now();
        let elapsed = now.duration_since(self.last_time).unwrap();
        if elapsed.as_secs() >= 10 {
            let generation_per_sec = self.generations_count as f64 / elapsed.as_secs_f64();
            let total_score = scores.iter().sum::<u64>();
            let average_score = total_score / scores.len() as u64;

            log::info!(
                "total {}, {} generations in 10 seconds, {:.5} generation/sec, average score {}, last best {}",
                total_generations_count,
                self.generations_count,
                generation_per_sec,
                average_score,
                scores[0]
            );
            self.last_time = now;
            self.generations_count = 0;
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let path = args().nth(1).expect("missing path");
    let frequency = args()
        .nth(2)
        .and_then(|v| read_frequency(Path::new(&v)).ok())
        .unwrap_or_default();
    let mut rng = StdRng::seed_from_u64(random());

    let mut bench = Bench::new();
    let mut playground = Playground::new(50, &mut rng, frequency);
    let mut best_score = u64::MAX;
    let mut best_keymap: Option<Keymap> = None;
    let mut last_scores: Vec<u64> = Vec::new();
    let conjunctions = read_4gram(Path::new(&path))?;
    let two_key_timing = TwoKeyTiming::load(Path::new("typing-time.html"))?;
    let scores = Arc::new(ConnectionScore::new(&two_key_timing));
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let mut no_update_long_time = false;

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("error setting handler");

    while running.load(Ordering::SeqCst) {
        let ret = playground.advance(&mut rng, &conjunctions, scores.clone(), no_update_long_time);

        if best_score > ret.0 {
            log::info!(
                "Got new best at {}! score: {}, current best: {} for evaluation:\n{:?}",
                playground.generation(),
                ret.0,
                ret.1,
                ret.1.key_combinations()
            );

            best_score = ret.0;
            best_keymap = Some(ret.1.clone());
        }

        if playground.generation() % 500 == 0 {
            // log::info!(
            //     "Long time no best at generation {}. last score: {}, last best score: {}, {} for evaluation:\n{:?}",
            //     playground.generation(),
            //     ret.0,
            //     best_score,
            //     best_keymap.clone().unwrap(),
            //     best_keymap.clone().unwrap().key_combinations()
            // );
            no_update_long_time = true;
        } else {
            no_update_long_time = false;
        }

        is_mutation_request(&mut last_scores, ret.0);
        bench.update(playground.generation(), &last_scores);
    }

    println!(
        "Score: {}, Best keymap: {} for evaluation:\n{:?}",
        best_score,
        best_keymap.clone().unwrap(),
        best_keymap.unwrap().key_combinations()
    );

    save_frequency(&playground.frequency_table());

    Ok(())
}

/// 突然変異を実行するかどうかを判定する。
///
/// 突然変異は、score更新前後で平均値が変わらない場合にrequestする
fn is_mutation_request(scores: &mut Vec<u64>, score: u64) -> bool {
    let previous_ave = if !scores.is_empty() {
        scores.iter().sum::<u64>() / scores.len() as u64
    } else {
        0
    };

    scores.insert(0, score);
    scores.truncate(1000);

    let current_ave = scores.iter().sum::<u64>() / scores.len() as u64;

    previous_ave == current_ave
}
