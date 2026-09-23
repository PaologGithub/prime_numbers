use std::{sync::mpsc::{self, Receiver, Sender}, thread, time::Instant};

use progress_bar::{
    Color, Style, finalize_progress_bar, inc_progress_bar, init_progress_bar, print_progress_bar_info, set_progress_bar_action,
};

fn calculate_thread(end: usize, sender: Sender<usize>) {
    // true if not prime
    let mut is_composite = vec![false; end + 1];

    for i in 2..=end {
        if !is_composite[i] {
            sender.send(i).unwrap();

            let mut j = i * i;
            while j <= end {
                is_composite[j] = true;
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
            let speed = prime as f64 / elapsed;
            
            set_progress_bar_action(
                &format!("{:.0} n/s", speed),
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

        inc_progress_bar();
    }
    
    finalize_progress_bar();
    primes
}

fn main() {
    let end: usize = 1_000_000;

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