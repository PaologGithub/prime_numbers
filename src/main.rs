use tokio::{
    task,
    time::Instant,
    sync::mpsc::{
        unbounded_channel,
        UnboundedReceiver,
        UnboundedSender
    }
};

use progress_bar::{
    Color, Style, finalize_progress_bar, init_progress_bar, print_progress_bar_info, set_progress_bar_action, set_progress_bar_progress,
};

const BATCH_SIZE: usize = 1000;

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

    #[inline]
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

    #[inline]
    pub fn get(&self, index: usize) -> bool {
        if index > self.len {
            panic!("index {} is greater than length {}", index, self.len);
        }

        let byte_index = index / 8;
        let bit_index = index % 8;
        (self.data[byte_index] & (1 << bit_index)) != 0
    }
}

async fn calculate_thread(end: usize, sender: UnboundedSender<Vec<usize>>) {
    // true if not prime
    let mut is_composite = BitVec::new(end + 1);
    let mut batch = Vec::with_capacity(BATCH_SIZE); // Batching

    for i in 2..=end {
        if !is_composite.get(i) {
            batch.push(i);
            
            if batch.len() >= BATCH_SIZE {
                // We can say here that the stdout thread won't finish, so won't drop tx, so unwrap isn't required
                let _ = sender.send(batch);
                batch = Vec::with_capacity(BATCH_SIZE);
            } 

            let mut j = i * i;
            while j <= end {
                is_composite.set(j, true);
                j += i;
            }
        }
    }

    if !batch.is_empty() {
        let _ = sender.send(batch);
    }
}

async fn stdout_thread(end: usize, mut receiver: UnboundedReceiver<Vec<usize>>) -> Vec<usize> {
    init_progress_bar(end);
    
    let mut primes: Vec<usize> = Vec::with_capacity((end as f64 / (end as f64).ln()) as usize);
    let mut start = Instant::now();

    while let Some(batch) = receiver.recv().await {
        let last = batch[batch.len() - 1];
        primes.extend(batch);
        
        let elapsed = start.elapsed().as_secs_f64();
        let speed = BATCH_SIZE as f64 / elapsed;
        
        set_progress_bar_action(
            &format!("{:.0} p/s", speed),
            Color::Blue,
            Style::Bold
        );
        
        print_progress_bar_info(
            "Found",
            &format!("{} primes (last: {})", primes.len(), last),
            Color::Green,
            Style::Bold,
        );
        
        start = Instant::now();

        set_progress_bar_progress(last);
    }
    
    finalize_progress_bar();
    primes
}

#[tokio::main]
async fn main() {
    let end: usize = 5_368_709_120;

    let (tx, rx): (UnboundedSender<Vec<usize>>, UnboundedReceiver<Vec<usize>>) = unbounded_channel();

    let _calculation_thread = task::spawn(async move {
        calculate_thread(end, tx).await;
    });

    let stdout_thread = task::spawn(async move {
        stdout_thread(end, rx).await
    });

    let primes = stdout_thread.await.unwrap();
    
    println!("Found {} primes (below {})", primes.len(), end);
}