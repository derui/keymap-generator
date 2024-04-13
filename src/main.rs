use std::{env::args, fs::File, path::Path};

use keymap::Keymap;
use rand::{rngs::StdRng, SeedableRng};
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

fn main() {
    let path = args().nth(1).expect("missing path");
    let mut rng = StdRng::seed_from_u64(9);

    let keymap = Keymap::generate(&mut rng);
    let conjunctions = read_4gram(&Path::new(&path));
    println!("{:?}", conjunctions)
}
