use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    env::args,
    fs::File,
    path::Path,
    sync::Arc,
    time::SystemTime,
};

use keymap::Keymap;
use rand::{random, rngs::StdRng, SeedableRng};
use score::Conjunction;

use crate::{connection_score::ConnectionScore, playground::Playground};

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
            let average_score = scores.iter().take(10).map(|v| v.0).sum::<u64>() / 10;

            log::info!(
                "total {}, {} generations in 60 seconds, {} generation/sec, highest average score {}",
                total_generations_count,
                self.generations_count,
                generation_per_sec,
                average_score
            );
            self.last_time = now;
            self.generations_count = 0;
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let path = args().nth(1).expect("missing path");
    let seed = args()
        .nth(2)
        .unwrap_or_else(|| random::<u64>().to_string())
        .parse::<u64>()?;
    let mut rng = StdRng::seed_from_u64(seed);

    let mut bench = Bench::new();
    let mut playground = Playground::new(84, &mut rng);
    let mut best_score = u64::MAX;
    let mut best_keymap: Option<Keymap> = None;
    let mut top_scores: BinaryHeap<Reverse<u64>> = BinaryHeap::new();
    let conjunctions = read_4gram(Path::new(&path))?;
    let scores = Arc::new(ConnectionScore::new());

    while !is_exit_score(&mut top_scores) {
        let ret = playground.advance(&mut rng, &conjunctions, scores.clone());

        if best_score > ret.0 {
            log::info!(
                "Got new best at {}, score is {}, current best is {} for evaluation:\n{:?}",
                playground.generation(),
                ret.0,
                ret.1,
                ret.1.key_combinations(&QWERTY)
            );

            best_score = ret.0;
            best_keymap = Some(ret.1);
        }

        top_scores.push(Reverse(ret.0));
        bench.update(playground.generation(), &top_scores);
    }

    println!(
        "Score: {}, Seed: {}, Best keymap: {} for evaluation:\n{:?}",
        best_score,
        seed,
        best_keymap.clone().unwrap(),
        best_keymap.unwrap().key_combinations(&QWERTY)
    );

    Ok(())
}

/// exitするかどうかを決定する。トップ5が同一のスコアであれば終了する
fn is_exit_score(score: &mut BinaryHeap<Reverse<u64>>) -> bool {
    if score.len() < 10 {
        return false;
    }

    let iter = score.iter().take(10).collect::<Vec<_>>();

    let base_score = iter.first().unwrap();

    let ret = iter.iter().all(|v| v == base_score);
    score.pop();
    ret
}
