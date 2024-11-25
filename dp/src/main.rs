use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Semaphore;

struct Philosopher {
    id: usize,
    semaphore: Arc<Semaphore>,
    stats: Arc<Mutex<HashMap<usize, (usize, usize)>>>, // (eats, thinks)
}

impl Philosopher {
    async fn think(&self) {
        {
            let mut stats = self.stats.lock().unwrap();
            let entry = stats.entry(self.id).or_default();
            entry.1 += 1; // Increment think count
        }

        println!("Philosopher {} is thinking...", self.id);
        tokio::time::sleep(Duration::from_millis(rand::random::<u64>() % 100)).await;
    }

    async fn eat(&self) {
        // Clone the Arc to increment the reference count, allowing the acquire_owned to work
        let semaphore_clone = self.semaphore.clone();
        let permit = semaphore_clone.acquire_owned().await.unwrap();

        {
            let mut stats = self.stats.lock().unwrap();
            let entry = stats.entry(self.id).or_default();
            entry.0 += 1; // Increment eat count
        }

        println!("Philosopher {} is eating...", self.id);
        tokio::time::sleep(Duration::from_millis(rand::random::<u64>() % 100)).await;

        // Permit is dropped here automatically, releasing the semaphore
    }
}

#[tokio::main]
async fn main() {
    let semaphore = Arc::new(Semaphore::new(2)); // Max 2 philosophers can eat simultaneously
    let stats = Arc::new(Mutex::new(HashMap::new()));

    let philosophers: Vec<_> = (0..5)
        .map(|id| Arc::new(Philosopher {
            id,
            semaphore: semaphore.clone(),
            stats: stats.clone(),
        }))
        .collect();

    let handles: Vec<_> = philosophers
        .iter()
        .map(|philosopher| {
            let philosopher = philosopher.clone(); // Clone the Arc to pass ownership to the task
            tokio::spawn(async move {
                loop {
                    philosopher.think().await;
                    philosopher.eat().await;
                }
            })
        })
        .collect();

    // Gracefully handle Ctrl+C to print statistics
    tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");

    println!("Cancelling philosophers...");
    for handle in handles {
        handle.abort(); // Cancel tasks
    }

    // Print statistics
    let stats = stats.lock().unwrap();
    for (id, (eats, thinks)) in stats.iter() {
        println!("Philosopher {}: {} eats, {} thinks", id, eats, thinks);
    }
}
