use std::{
    cmp::Reverse, collections::{BinaryHeap, HashMap}, env::args, fs::{self, File}, io::{Read, Write}, path::Path, sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    }, time::SystemTime
};

use frequency_table::FrequencyTable;
use keymap::Keymap;
use postcard::{from_bytes, to_allocvec};
use rand::{random, rngs::StdRng, SeedableRng};
use score::Conjunction;

use crate::{connection_score::{CharFrequency, ConnectionScore}, playground::Playground};

mod char_def;
mod connection_score;
mod frequency_table;
mod key_def;
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

    let char_position_map: HashMap<char, usize> = char_def::all_chars()
        .into_iter()
        .enumerate()
        .map(|(idx, v)| (v, idx))
        .collect();

    for result in rdr.records() {
        // The iterator yields Result<StringRecord, Error>, so we check the
        // error here.
        let record = result?;
        let text: String = record.get(0).unwrap().to_string();
        let appearances: u32 = record.get(1).unwrap().parse()?;
        conjunctions.push(Conjunction {
            text: text
                .chars()
                .filter_map(|v| char_position_map.get(&v))
                .cloned()
                .collect(),
            appearances,
        });
    }

    Ok(conjunctions)
}

fn read_2gram(path: &Path) -> anyhow::Result<Vec<Conjunction>> {
    let mut conjunctions = Vec::new();
    let file = File::open(path).unwrap();

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(&file);

    let char_position_map: HashMap<char, usize> = char_def::all_chars()
        .into_iter()
        .enumerate()
        .map(|(idx, v)| (v, idx))
        .collect();

    for result in rdr.records() {
        // The iterator yields Result<StringRecord, Error>, so we check the
        // error here.
        let record = result?;
        let Some(text) = record
            .get(2)
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string())
        else {
            break;
        };
        let appearances: u32 = record.get(3).unwrap().parse()?;
        conjunctions.push(Conjunction {
            text: text
                .chars()
                .filter_map(|v| char_position_map.get(&v))
                .cloned()
                .collect(),
            appearances,
        });
    }

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

const QWERTY: [[char; 10]; 3] = [
    ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
    ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';'],
    ['z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/'],
];

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

    fn update(&mut self, total_generations_count: u64, scores: &BinaryHeap<Reverse<u64>>) {
        self.generations_count += 1;
        let now = SystemTime::now();
        let elapsed = now.duration_since(self.last_time).unwrap();
        if elapsed.as_secs() > 60 {
            let generation_per_sec = self.generations_count as f64 / elapsed.as_secs_f64();
            let scores = scores.iter().cloned().map(|v| v.0);
            let score_len = scores.len();
            let average_score = scores.sum::<u64>() / score_len as u64;

            log::info!(
                "total {}, {} generations in 60 seconds, {:.5} generation/sec, highest average score {}",
                total_generations_count,
                self.generations_count,
                generation_per_sec,
                average_score,
            );
            self.last_time = now;
            self.generations_count = 0;
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let path = args().nth(1).expect("missing path");
    let path_2gram = args().nth(2).expect("missing path");
    let frequency = args()
        .nth(3)
        .and_then(|v| read_frequency(Path::new(&v)).ok())
        .unwrap_or(FrequencyTable::new());
    let mut rng = StdRng::seed_from_u64(random());

    let mut bench = Bench::new();
    let mut playground = Playground::new(50, &mut rng, frequency);
    let mut best_score = u64::MAX;
    let mut best_keymap: Option<Keymap> = None;
    let mut top_scores: BinaryHeap<Reverse<u64>> = BinaryHeap::new();
    let conjunctions = read_4gram(Path::new(&path))?;
    let conjunctions_2gram = read_2gram(Path::new(&path_2gram))?;
    let char_frequency = CharFrequency::read(Path::new(&path_2gram))?;
    let scores = Arc::new(ConnectionScore::new());
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("error setting handler");

    while !is_exit_score(&mut top_scores) && running.load(Ordering::SeqCst) {
        let ret = playground.advance(&mut rng, &conjunctions, scores.clone(), &conjunctions_2gram, &char_frequency);

        if best_score > ret.0 {
            log::info!(
                "Got new best at {}, score is {}, current best is {} for evaluation:\n{:?}",
                playground.generation(),
                ret.0,
                ret.1,
                ret.1.key_combinations(&QWERTY)
            );

            best_score = ret.0;
            best_keymap = Some(ret.1.clone());
        }

        top_scores.push(Reverse(ret.0));
        bench.update(playground.generation(), &top_scores);
    }

    println!(
        "Score: {}, Best keymap: {} for evaluation:\n{:?}",
        best_score,
        best_keymap.clone().unwrap(),
        best_keymap.unwrap().key_combinations(&QWERTY)
    );

    save_frequency(&playground.frequency_table());

    Ok(())
}

/// exitするかどうかを決定する。トップ10が同一のスコアであれば終了する
fn is_exit_score(score: &mut BinaryHeap<Reverse<u64>>) -> bool {
    if score.len() < 10 {
        return false;
    }

    let iter = score.iter().take(10).collect::<Vec<_>>();

    let base_score = iter.first().unwrap();

    let ret = iter.iter().all(|v| v == base_score);

    if score.len() > 10000 {
        score.clear();
    }
    ret
}
