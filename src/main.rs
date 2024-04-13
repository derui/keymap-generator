use std::{env::args, fs::File, path::Path};

use keymap::Keymap;
use rand::{random, rngs::StdRng, SeedableRng};
use score::Conjunction;

mod char_def;
mod key;
mod keymap;
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
    env_logger::Builder::new()
        .target(env_logger::Target::Stderr)
        .init();

    let path = args().nth(1).expect("missing path");
    let seed = args()
        .nth(2)
        .unwrap_or_else(|| random::<u64>().to_string())
        .parse::<u64>()?;
    let mut rng = StdRng::seed_from_u64(seed);

    let mut keymap = Keymap::generate(&mut rng);
    let mut best_score = u64::MAX;
    let mut best_keymap: Option<Keymap> = None;
    let conjunctions = read_4gram(&Path::new(&path))?;
    log::info!("initial keymap: {}", keymap);

    while keymap.generation() < 5000 {
        let new_keymap = keymap.mutate(&mut rng);

        // 条件を満たす場合のみ評価し、次の世代を構成できるものとする
        if new_keymap.meet_requirements() {
            let score = score::evaluate(&conjunctions, &new_keymap);

            if best_score > score {
                log::info!(
                    "updated {} -> {} at generation {}",
                    best_score,
                    score,
                    new_keymap.generation()
                );
                best_score = score;
                best_keymap = Some(new_keymap.clone());
            } else {
                log::info!(
                    "Get score {}, but best is {} at generation {}",
                    score,
                    best_score,
                    new_keymap.generation()
                );
            }
            keymap = new_keymap;
        }

        if keymap.generation() % 1000 == 0 {
            log::info!("Processed generation: {}", keymap.generation());
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
