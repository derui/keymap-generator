use std::{env::args, fs::File, path::Path};

use keymap::Keymap;
use rand::{random, rngs::StdRng, SeedableRng};
use score::Conjunction;

use crate::playground::Playground;

mod char_def;
mod key;
mod keymap;
mod playground;
mod score;

fn read_4gram(path: &Path) -> anyhow::Result<Vec<Conjunction>> {
    let mut conjunctions = Vec::new();
    let file = File::open(path).unwrap();

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(&file);

    for result in rdr.records() {
        // The iterator yields Result<StringRecord, Error>, so we check the
        // error here.
        let record = result?;
        let text: String = record.get(0).unwrap().to_string();
        let appearances: u32 = record.get(1).unwrap().parse()?;
        conjunctions.push(Conjunction { text, appearances });
    }

    Ok(conjunctions)
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let path = args().nth(1).expect("missing path");
    let seed = args()
        .nth(2)
        .unwrap_or_else(|| random::<u64>().to_string())
        .parse::<u64>()?;
    let mut rng = StdRng::seed_from_u64(seed);

    let mut playground = Playground::new(255, &mut rng);
    let mut best_score = u64::MAX;
    let mut best_keymap: Option<Keymap> = None;
    let conjunctions = read_4gram(Path::new(&path))?;

    while playground.generation() < 5000 {
        let ret = playground.advance(&mut rng, &conjunctions);
        best_score = ret.0;
        best_keymap = Some(ret.1);

        if playground.generation() % 100 == 0 {
            log::info!(
                "Processed generation: {}, current best is {}",
                playground.generation(),
                best_keymap.clone().unwrap()
            );
        }
    }

    println!(
        "Score: {}, Seed: {}, Best keymap: {}",
        best_score,
        seed,
        best_keymap.unwrap()
    );

    Ok(())
}
