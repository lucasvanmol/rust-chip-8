use std::{
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

#[derive(PartialEq, Debug)]
pub enum Register {
    Vx(u8),
}

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct Registers {
    pub PC: usize,    // Program Counter (u16)
    pub SP: u8,       // Stack Pointer
    pub I: u16,       // I register
    pub Vx: [u8; 16], // General Purpose Vx registers
    DT: Arc<AtomicU8>,
    ST: Arc<AtomicU8>, // Sound & Timer registers
}

impl Registers {
    pub fn new() -> Self {
        let r = Registers {
            PC: 0x200,
            SP: 0,
            I: 0,
            Vx: [0; 16],
            DT: Arc::new(AtomicU8::new(0)),
            ST: Arc::new(AtomicU8::new(0)),
        };
        r.init();
        r
    }

    fn spawn_timer_thread(lock: Arc<AtomicU8>) {
        thread::spawn(move || loop {
            thread::sleep(Duration::from_nanos(16_666_667));
            if lock.load(Ordering::Relaxed) != 0 {
                lock.fetch_sub(1, Ordering::SeqCst);
            }
        });
    }

    pub fn init(&self) {
        let dt_lock = self.DT.clone();
        let st_lock = self.ST.clone();

        Registers::spawn_timer_thread(dt_lock);
        Registers::spawn_timer_thread(st_lock);
    }

    pub fn is_dt_active(&self) -> bool {
        self.get_dt() != 0
    }

    pub fn get_dt(&self) -> u8 {
        self.DT.load(Ordering::Relaxed)
    }

    pub fn set_dt(&self, val: u8) {
        self.DT
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(val))
            .unwrap();
    }

    pub fn is_st_active(&self) -> bool {
        self.get_st() != 0
    }

    pub fn get_st(&self) -> u8 {
        self.ST.load(Ordering::Relaxed)
    }

    pub fn set_st(&self, val: u8) {
        self.ST
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(val))
            .unwrap();
    }
}
