use std::fs;

use crate::nix::FlakeLock;

mod nix;

fn main() {
    let flake_lock_json = fs::read_to_string("flake.lock").expect("Couldn't load flake.lock");
    let flake_lock: FlakeLock =
        serde_json::from_str(&flake_lock_json).expect("Couldn't deserialize flake.lock");

    println!("{:?}", flake_lock);
}
