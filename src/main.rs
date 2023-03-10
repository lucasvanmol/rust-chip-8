mod chip8;

use argh::FromArgs;
use chip8::CHIP8;

#[derive(FromArgs)]
/// Chip-8 Emulator
struct Args {
    #[argh(positional)]
    /// filename of the Chip-8 cartridge binary
    filename: String,
}

fn main() {
    let filename = argh::from_env::<Args>().filename;
    let mut chip8 = CHIP8::new();

    match chip8.load(&filename) {
        Ok(_) => chip8.run(),
        Err(e) => eprintln!("Could not open file `{filename}`: {e}"),
    }
}
