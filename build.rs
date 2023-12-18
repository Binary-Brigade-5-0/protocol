use std::io::Write;
use std::{fs::File, path::Path};

use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let outdir = std::env::var("OUT_DIR")?;
    let outdir = Path::new(&outdir).join("jwt-key.out");

    let mut key_bytes: [u8; 1024] = [0; 1024];
    let mut seed: <ChaCha20Rng as SeedableRng>::Seed = Default::default();
    thread_rng().fill(&mut seed);

    let mut generator = ChaCha20Rng::from_seed(seed);
    generator.fill(&mut key_bytes);

    let mut file = File::create(outdir)?;
    file.write_all(&key_bytes)?;

    // trigger recompilation when a new migration is added
    println!("cargo:rerun-if-changed=migrations");
    Ok(())
}
