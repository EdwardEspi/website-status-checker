# Website Status Checker

A concurrent website-monitoring tool written in Rust. This command-line utility checks the availability of multiple websites in parallel, with configurable options for worker threads, timeouts, and retries.

## Features
- **Concurrent Website Checking**:  
  Uses a fixed worker-thread pool to process multiple URLs concurrently. The number of worker threads is configurable via the `--workers` flag.
- **Retries for Failed Requests**:  
  Automatically retries failed requests up to a specified number of times using the `--retries` flag, with a 100ms delay between retries.
- **Timeouts**:  
  Each HTTP request has a configurable timeout using the `--timeout` flag.
- **Input Flexibility**:  
  Accepts URLs directly as command-line arguments or from a file using the `--file` flag. Blank lines and lines starting with `#` are ignored.
- **Live Output**:  
  Prints a human-readable summary line to stdout for each URL as soon as it is checked.
- **Manual JSON Output**:  
  Results are written to a `status.json` file, generated manually using only the Rust standard library.
- **Error Handling**:  
  Gracefully handles invalid URLs, timeouts, and other HTTP errors without crashing.
- **Unit Tests**:  
  Includes tests for the `check_website` function to ensure reliability for success, failure, and retry scenarios.

## Build Instructions
To build the project in release mode:
```bash
cargo build --release
```

## Usage Examples
Check URLs from a file:
```bash
cargo run --release -- --file sites.txt --workers 4 --timeout 10 --retries 2
```
Check a single URL:
```bash
cargo run -- https://www.rust-lang.org
```
Check a mix of file and command-line URLs:
```bash
cargo run -- --file sites.txt https://www.textcompactor.com --workers 4
```

## JSON Output Fields
Each entry in `status.json` contains:
- `url`: The URL that was checked.
- `status`: An object with either `"Ok": <code>` for HTTP status or `"Err": "<error message>"`.
- `response_time_ms`: The response time in milliseconds.
- `timestamp`: The timestamp when the check completed.

## What I Implemented
1. **Worker Thread Pool**:  
   A fixed pool of worker threads processes URLs concurrently. The number of threads defaults to the number of logical CPU cores but can be customized using the `--workers` flag.
2. **Retries**:  
   Retry logic in the `check_website` function, with a 100ms delay between retries.
3. **Timeouts**:  
   Each HTTP request has a configurable timeout (`--timeout`).
4. **Input Handling**:  
   URLs can be provided as command-line arguments or read from a file (`--file`).
5. **Live Output**:  
   Prints a summary line to stdout for each URL as soon as it is checked.
6. **Manual JSON Output**:  
   Results are written to `status.json` using only the Rust standard library.
7. **Unit Tests**:  
   Tests for the `check_website` function: success, failure, and retry logic.
8. **Error Handling**:  
   Handles invalid URLs, timeouts, and other HTTP errors gracefully.

## Example Workflow
1. **Input File**:  
   The program supports reading URLs from a file, ignoring blank lines and comments.
2. **Command**:  
   Run the program with a combination of file-based and command-line URLs, specifying the number of workers, timeout, and retries:
   ```bash
   cargo run --release -- --file sites.txt --workers 4 --timeout 10 --retries 2
   ```
3. **Output**:  
   The program writes the results to a `status.json` file. Each entry includes the URL, HTTP status code or error message, response time in milliseconds, and timestamp.
4. **Unit Tests**:  
   Run the tests to validate the success, failure, and retry logic:
   ```bash
   cargo test
   ```

## Implementation Details
- **Concurrency**:  
  Uses Rust's `std::thread` and `std::sync` modules to create a fixed worker-thread pool. A channel (`mpsc::channel`) is used to distribute URLs to worker threads.
- **HTTP Requests**:  
  Uses the `reqwest` crate (with the `blocking` feature) to perform HTTP requests, with timeout and retry logic.
- **Manual JSON Generation**:  
  Generates the `status.json` file manually using only the Rust standard library, as required by the assignment.
- **Error Handling**:  
  Errors are handled gracefully, and meaningful messages are provided for issues like invalid URLs, timeouts, or missing input files.

## Bonus Features

- **Summary Statistics**:  
  After each run, the program prints the minimum, maximum, and average response times for all successful requests. This provides a quick overview of the performance of the checked websites.

## Summary
This project demonstrates the use of Rust's concurrency features, error handling, and manual JSON serialization. It is a robust and configurable tool for monitoring website availability, designed to handle real-world scenarios like timeouts, retries, and invalid inputs.