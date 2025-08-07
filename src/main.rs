use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, Seek, SeekFrom, Write},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
    env::var,
};

const LOG_FILE_PATH: &str = "stats.log";
const LOG_UPDATE_INTERVAL_MS: u64 = 1000; // Update log every 1000ms (1 second)

/// The worker task that repeatedly processes the user's string.
/// This will run in multiple threads.
fn processor_task(
    shared_string: Arc<String>,
    counter: Arc<AtomicUsize>,
    running: Arc<AtomicBool>,
) {
    while running.load(Ordering::Relaxed) {
        // Clone the string (computationally cheap) and discard immediately
        let _ = shared_string.clone();
        counter.fetch_add(1, Ordering::Relaxed);
    }
}

/// The main task for periodically logging statistics.
fn logger_task(
    counter: Arc<AtomicUsize>,
    start_time: Instant,
    log_file_mutex: Arc<Mutex<File>>,
    running: Arc<AtomicBool>,
    update_interval: Duration,
) {
    println!(
        "Logger thread started. Updating {} every {:?}.",
        LOG_FILE_PATH, update_interval
    );

    let check_interval = Duration::from_millis(100);
    let mut last_log_time = Instant::now();

    while running.load(Ordering::Relaxed) {
        if !running.load(Ordering::Relaxed) {
            break;
        }

        if last_log_time.elapsed() >= update_interval {
            let processed_count = counter.load(Ordering::Relaxed);
            let elapsed_time = start_time.elapsed();
            let elapsed_seconds = elapsed_time.as_secs_f64();

            let average_speed = if elapsed_seconds > 0.0 {
                processed_count as f64 / elapsed_seconds
            } else {
                0.0
            };

            // Format with precise float formatting
            let stats_string = format!(
                "Processed: {:<15} | Elapsed: {:.2}s | Speed: {:.2}/s",
                processed_count,
                elapsed_seconds,
                average_speed
            );

            // Truncate to exact 100 characters (prevents file resizing)
            let padded_stats_string = stats_string.chars().take(100).collect::<String>();

            // Write to log file (truncates file to 100 chars)
            {
                let mut log_file = log_file_mutex.lock().expect("Failed to lock log file");
                if log_file.seek(SeekFrom::Start(0)).is_ok() {
                    log_file.write_all(padded_stats_string.as_bytes()).expect("Failed to write to log");
                    log_file.flush().expect("Failed to flush log");
                }
            }

            // Print to console (same content as log file)
            println!("{}", padded_stats_string);

            last_log_time = Instant::now();
        }

        thread::sleep(check_interval);
    }
    println!("Logger thread stopping.");
}

fn main() -> std::io::Result<()> {
    println!("Starting high-speed string repeater program...");

    // --- Get User Input String ---
    let user_string: String;
    loop {
        print!("Enter the string to repeat: ");
        io::stdout().flush()?; // Ensure prompt is displayed

        let mut buffer = String::new();
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        
        match handle.read_line(&mut buffer) {
            Ok(0) => {
                println!("\nEOF detected. Exiting.");
                return Ok(()); // Exit if no input given
            }
            Ok(_) => {
                let trimmed = buffer.trim();
                if trimmed.is_empty() {
                    println!("Input cannot be empty. Please try again.");
                    continue;
                }
                user_string = trimmed.to_string();
                break;
            }
            Err(e) => {
                eprintln!("\nError reading input: {}", e);
                return Err(e);
            }
        }
    }
    println!("Repeating the string: \"{}\"", user_string);
    // --- End Get User Input String ---

    // Wrap the user's string in an Arc for safe sharing
    let shared_user_string = Arc::new(user_string);

    // Determine number of worker threads
    let num_worker_threads = thread::available_parallelism()?.get();
    println!(
        "Will spawn {} worker threads to repeat the string.",
        num_worker_threads
    );
    println!("Statistics logged to {} every {}ms.", LOG_FILE_PATH, LOG_UPDATE_INTERVAL_MS);
    println!("Press Ctrl+C to stop.");

    // Shared state: counter and running flag
    let processed_counter = Arc::new(AtomicUsize::new(0));
    let running_flag = Arc::new(AtomicBool::new(true));

    // Record start time (after getting user input)
    let start_time = Instant::now();

    // Log File Setup
    let log_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(LOG_FILE_PATH)?;
    let log_file_mutex = Arc::new(Mutex::new(log_file));

    // --- Spawn Threads ---
    let mut thread_handles: Vec<JoinHandle<()>> = Vec::with_capacity(num_worker_threads + 1);

    // Spawn Worker Threads
    println!("Spawning worker threads...");
    for _ in 0..num_worker_threads {
        let processor_string_clone = Arc::clone(&shared_user_string);
        let processor_counter_clone = Arc::clone(&processed_counter);
        let processor_running_clone = Arc::clone(&running_flag);
        
        let handle = thread::spawn(move || {
            processor_task(processor_string_clone, processor_counter_clone, processor_running_clone);
        });
        thread_handles.push(handle);
    }
    println!("All worker threads spawned.");

    // Spawn Logger Thread
    let logger_counter_clone = Arc::clone(&processed_counter);
    let logger_file_clone = Arc::clone(&log_file_mutex);
    let logger_running_clone = Arc::clone(&running_flag);
    let log_interval = Duration::from_millis(LOG_UPDATE_INTERVAL_MS);

    let logger_handle = thread::spawn(move || {
        logger_task(
            logger_counter_clone,
            start_time,
            logger_file_clone,
            logger_running_clone,
            log_interval,
        );
    });
    thread_handles.push(logger_handle);
    // --- End Spawn Threads ---

    // Graceful Shutdown Handling
    let running_flag_ctrlc = Arc::clone(&running_flag);
    ctrlc::set_handler(move || {
        println!("\nCtrl+C received. Shutting down gracefully...");
        running_flag_ctrlc.store(false, Ordering::Relaxed);
    }).expect("Error setting Ctrl-C handler");

    // Wait for all threads to finish
    println!("Waiting for threads to complete...");
    for handle in thread_handles {
        handle.join().expect("A worker thread panicked");
    }

    // Final statistics output
    let final_count = processed_counter.load(Ordering::Relaxed);
    let total_time = start_time.elapsed();
    let avg_speed = if total_time.as_secs_f64() > 0.0 {
        final_count as f64 / total_time.as_secs_f64()
    } else {
        0.0
    };

    println!("\n--- Program Finished ---");
    println!("Total repetitions processed: {}", final_count);
    println!("Total time elapsed: {:?}", total_time);
    println!("Average speed: {:.2} repetitions/s", avg_speed);
    println!("Log file saved to: {}", LOG_FILE_PATH);

    Ok(())
}
