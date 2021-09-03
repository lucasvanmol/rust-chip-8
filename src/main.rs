mod chip8;

use std::env;
use chip8::CHIP8;

fn main() {
    let mut args = env::args();
    let filename = args.nth(1).expect("Missing required argument: filename");
    let mut chip8 = CHIP8::new();
    chip8.load(filename).unwrap();
    chip8.run();
}
