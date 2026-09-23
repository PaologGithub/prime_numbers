use std::{sync::mpsc::{self, Receiver, Sender}, thread, time::Instant, usize};

use progress_bar::{
    Color, Style, finalize_progress_bar, init_progress_bar, print_progress_bar_info, set_progress_bar_action, set_progress_bar_progress,
};

/// To store each `bool` as a single bit
struct BitVec {
    /// Each element stores 8 bool
    data: Vec<u8>,
    len: usize
}

impl BitVec {
    pub fn new(len: usize) -> Self {
        let byte_len = (len + 7) / 8;
        Self {
            data: vec![0u8; byte_len],
            len
        }
    }

    pub fn set(&mut self, index: usize, value: bool) {
        if index > self.len {
            panic!("index {} is greater than length {}", index, self.len);
        }

        let byte_index = index / 8;
        let bit_index = index % 8;
        if value {
            self.data[byte_index] |= 1 << bit_index;
        } else {
            self.data[byte_index] &= !(1 << bit_index);
        }
    }

    pub fn get(&self, index: usize) -> bool {
        if index > self.len {
            panic!("index {} is greater than length {}", index, self.len);
        }

        let byte_index = index / 8;
        let bit_index = index % 8;
        (self.data[byte_index] & (1 << bit_index)) != 0
    }
}

fn calculate_thread(end: usize, sender: Sender<usize>) {
    // true if not prime
    let mut is_composite = BitVec::new(end + 1);

    for i in 2..=end {
        if !is_composite.get(i) {
            sender.send(i).unwrap();

            let mut j = i * i;
            while j <= end {
                is_composite.set(j, true);
                j += i;
            }
        }
    }

    drop(sender); // Kill the channel
}

fn stdout_thread(end: usize, receiver: Receiver<usize>) -> Vec<usize> {
    init_progress_bar(end);
    
    let mut primes: Vec<usize> = Vec::with_capacity((end as f64 / (end as f64).ln()) as usize);
    let mut start = Instant::now();

    while let Ok(prime) = receiver.recv() {
        primes.push(prime);
        
        if primes.len() % 1000 == 0 {
            let elapsed = start.elapsed().as_secs_f64();
            let speed = 1000.0 / elapsed;
            
            set_progress_bar_action(
                &format!("{:.0} p/s", speed),
                Color::Blue,
                Style::Bold
            );
            
            print_progress_bar_info(
                "Found",
                &format!("{} primes (last: {})", primes.len(), prime),
                Color::Green,
                Style::Bold,
            );
            
            start = Instant::now();
        }

        set_progress_bar_progress(prime);
    }
    
    finalize_progress_bar();
    primes
}

fn main() {
    let end: usize = 5_368_709_120;

    let (tx, rx): (Sender<usize>, Receiver<usize>) = mpsc::channel();

    let calculation_thread = thread::spawn(move || {
        calculate_thread(end, tx);
    });

    let stdout_thread = thread::spawn(move || {
        stdout_thread(end, rx)
    });

    let primes = stdout_thread.join().unwrap();
    
    println!("Found {} primes (below {})", primes.len(), end);
}