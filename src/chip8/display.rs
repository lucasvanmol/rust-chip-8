use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use minifb::{Key, Scale, Window, WindowOptions};

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

type Buffer = [u32; WIDTH * HEIGHT];

// to do :
// update buffer is super slow. maybe only send buffer update every few hz? -> set fps

pub struct Display {
    screen: Arc<RwLock<Buffer>>,
    buffer: Buffer,
    pub handle: JoinHandle<()>,
    keys_pressed: Arc<RwLock<Vec<Key>>>,
}

impl Display {
    pub fn update_buffer(&self) {
        // TODO: add dynamic sleep to get consistent fps, and buffer key inputs.
        // consider using Mutex instead of RwLock
        thread::sleep(Duration::from_micros(1));
        *self.screen.write().unwrap() = self.buffer;
    }

    pub fn init() -> Self {
        let screen = Arc::new(RwLock::new([0; WIDTH * HEIGHT]));
        let screen_lock = screen.clone();
        let buffer = [0; WIDTH * HEIGHT];

        let keys_pressed = Arc::new(RwLock::new(vec![]));
        let key_buffer = keys_pressed.clone();

        let handle = thread::spawn(move || {
            let mut opts = WindowOptions::default();
            opts.scale = Scale::X16;

            let mut window = Window::new("Test - ESC to exit", WIDTH, HEIGHT, opts).unwrap();

            window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

            while window.is_open() && !window.is_key_down(Key::Escape) {
                match screen_lock.try_read() {
                    Ok(gaurd) => window.update_with_buffer(&*gaurd, WIDTH, HEIGHT).unwrap(),
                    Err(_) => window.update(),
                };

                if let Some(keys) = window.get_keys() {
                    *keys_pressed.write().unwrap() = keys.clone();
                }

                // Allow the buffer to be updated
                thread::sleep(Duration::from_micros(1));
            }
        });

        Display {
            screen,
            buffer,
            handle,
            keys_pressed: key_buffer,
        }
    }

    pub fn is_window_open(&self) -> bool {
        !self.handle.is_finished()
    }

    pub fn get_key_down(&self) -> Option<Key> {
        self.keys_pressed.read().unwrap().get(0).map(Key::clone)
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        self.keys_pressed.read().unwrap().contains(&key)
    }

    pub fn clear(&mut self) {
        self.buffer = [0; WIDTH * HEIGHT];
    }

    fn to_index(x: usize, y: usize) -> usize {
        let y = y % HEIGHT;
        let x = x % WIDTH;
        WIDTH * y + x
    }

    pub fn set_pixels(&mut self, x: u8, y: u8, bytes: &[u8]) -> bool {
        let mut collision = false;
        let num_bytes = bytes.len();
        let slice = &mut self.buffer;

        for j in 0..num_bytes {
            // For every bit in byte, check if 1
            for i in 0..8 {
                let filter: u8 = 0b10000000 >> i;
                if bytes[j] & filter == filter {
                    // If so, XOR with buffer value, and track collision
                    let index = Display::to_index(x as usize + i, y as usize + j); // % (WIDTH * HEIGHT);
                    if slice[index] == u32::MAX {
                        collision = true;
                        slice[index] = 0;
                    } else {
                        slice[index] = u32::MAX;
                    }
                }
            }
        }

        collision
    }
}
