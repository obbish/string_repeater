# String Repeater — High-Speed Repetition Utility

## Overview

This Rust program is designed to take a specific string provided by the user and then repeatedly "process" (clone and immediately discard) that exact string in memory at the highest possible speed. It leverages multi-threading to utilize available CPU cores, maximizing the repetition rate. The primary purpose is to demonstrate and benchmark high-throughput, CPU-bound processing of given data, focusing on memory allocation/copy/deallocation cycles.

The program continuously tracks the number of repetitions, elapsed time, and the average repetition speed (repetitions per second), logging these statistics to a file every second.

## Features

* **User-Defined Target String:** Accepts the specific string to be processed from user input at startup.
* **Multi-Core Processing:** Spawns multiple worker threads (based on available CPU parallelism) to maximize repetition throughput.
* **High-Speed Repetition:** Focuses computational effort on rapidly cloning and discarding the user's string data in memory.
* **Performance Statistics:** Tracks total repetitions, elapsed time, and average speed.
* **Real-time Logging:** Updates a fixed-size log file (`stats.log`) every second with the current statistics, overwriting previous content.
* **Graceful Shutdown:** Captures `Ctrl+C` signal to stop cleanly and display final results.

### Prerequisites

* **Rust Toolchain:** You need `rustc` (the compiler) and `cargo` (the build tool and package manager) installed. If you don't have them, install Rust from [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).

## Setup and Compilation

1.  **Get the Code:** Clone the repository or download the source files (`src/main.rs` and `Cargo.toml`).
2.  **Navigate:** Open your terminal or command prompt and change directory into the project's root folder (where `Cargo.toml` is located).
3.  **Build Release Version:** Compile the program with optimizations for maximum performance.
    ```bash
    cargo build --release
    ```
    *__Note:__ Building without the `--release` flag will result in significantly lower performance.*

## Running the Program

1.  **Execute:** Run the compiled binary from the project's root directory:
    ```bash
    ./target/release/string_repeater
    ```
    *(On Windows, you might run `.\target\release\string_repeater.exe`)*

2.  **Input String:** The program will prompt you:
    ```
    Enter the string to repeat:
    ```
    Type the string you want the program to process repeatedly and press `Enter`.

3.  **Processing:** The program will confirm the string and start the high-speed repetition process using multiple threads. You should observe high CPU usage while it's running.

4.  **Monitor Log:** Check the `stats.log` file created in the same directory. It will update every second with the latest statistics.

## Stopping the Program

* Press `Ctrl+C` in the terminal where the program is running.
* The program will detect the signal, stop the worker threads gracefully, and print a final summary of the total repetitions and average speed to the console before exiting.

## Output

* **Console:**
    * Initial startup messages and prompts for input.
    * Confirmation of the string being processed and the number of threads spawned.
    * Messages during graceful shutdown (`Ctrl+C`).
    * A final summary upon exit, showing total repetitions, total time elapsed, and the overall average speed.
* **Log File (`stats.log`):**
    * Located in the same directory as the executable.
    * Contains a single line of text that is overwritten every second (maintaining a fixed file size after the first write).
    * Format: `Processed: [COUNT] | Elapsed: [TIME]s | Speed: [SPEED]/s`
    * Example:
        ```
        Processed: 238010593      | Elapsed: 10.00      s | Speed: 23801059.30   /s
        ```

## Configuration (Optional)

Basic parameters can be adjusted by modifying constants at the top of the `src/main.rs` file:

* `LOG_UPDATE_INTERVAL_MS`: How often (in milliseconds) the log file is updated (default: 1000).
* `LOG_LINE_WIDTH`: The fixed width (padding) for the log file line to ensure consistent size (default: 100).

If you change these constants, you will need to recompile the program using `cargo build --release`.
