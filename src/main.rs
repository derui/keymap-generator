use keymap::Keymap;
use rand::{rngs::StdRng, SeedableRng};

mod char_def;
mod key;
mod keymap;
mod score;

fn main() {
    let mut rng = StdRng::seed_from_u64(9);

    let keymap = Keymap::generate(&mut rng);
    println!("{:?}", keymap);
}
