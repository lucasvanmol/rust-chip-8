use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError};
use std::rc::Rc;

use minifb::{Key, Scale, Window, WindowOptions};

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

type Buffer = [u32; WIDTH * HEIGHT];

// to do :
// update buffer is super slow. maybe only send buffer update every few hz? -> set fps

pub struct Display {
    buffer: Buffer,
    pub handle: JoinHandle<()>,
    buf_tx: SyncSender<Buffer>,
    end_rx: Receiver<bool>,
    keys_pressed: Arc<RwLock<Vec<Key>>>
}


impl Display {

    pub fn update_buffer(&self) {
        self.buf_tx.send(self.buffer).unwrap();
    }

    pub fn init() -> Self {
        let buffer = [0; WIDTH * HEIGHT];

        let (buf_tx, buf_rx) : (SyncSender<Buffer>, Receiver<Buffer>) = mpsc::sync_channel(1);
        let (key_tx, key_rx) = mpsc::channel();
        let (end_tx, end_rx) = mpsc::sync_channel(0);

        let handle =  thread::spawn(move || {
            let mut opts = WindowOptions::default();
            opts.scale = Scale::X16;
        
            let mut window = Window::new(
                "Test - ESC to exit",
                WIDTH,
                HEIGHT,
                opts,
            ).unwrap();


            let mut keys: Rc<Vec<Key>> = Rc::new(vec![]); 
            while window.is_open() && !window.is_key_down(Key::Escape) {
                match buf_rx.try_recv() {
                    Ok(buffer) => {
                        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
                    }
                    Err(_) => window.update(),
                }
                
                let new_keys = Rc::new(window.get_keys().unwrap_or(vec![]));
                if keys != new_keys {
                    keys = Rc::clone(&new_keys);
                    key_tx.send((*keys).clone()).unwrap();
                }
            }
            // Send end signal & flush out buf channel so that buf_tx.send does not hang
            end_tx.send(true).unwrap();
            buf_rx.try_recv().unwrap();
        });

        let keys_pressed = Arc::new(RwLock::new(vec![]));
        let key_buffer = keys_pressed.clone();

        thread::spawn(move || {
            loop {
                match key_rx.recv() {
                    Ok(keys) => *keys_pressed.write().unwrap() = keys,
                    Err(_) => break,
                }
            }
        });

        Display {
            buffer,
            handle,
            buf_tx,
            end_rx,
            keys_pressed: key_buffer,
        }
    }

    pub fn is_window_open(&self) -> bool {
        self.end_rx.try_recv() != Err(TryRecvError::Disconnected)
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
                    let index = Display::to_index(x as usize + i, y as usize + j);// % (WIDTH * HEIGHT);
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
