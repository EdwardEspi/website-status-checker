use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use reqwest::blocking::Client;

struct WebsiteStatus {
    url: String,
    status: Result<u16, String>, // HTTP status code or error message
    response_time_ms: u128,      // Response time in milliseconds
    timestamp: String,           // Timestamp of the check
}

fn main() {
    // Collect command-line arguments
    let args: Vec<String> = env::args().collect();

    // If no arguments are provided, print usage and exit
    if args.len() < 2 {
        print_usage();
        std::process::exit(2);
    }

    // Initialize default values
    let mut file_path: Option<String> = None;
    let mut urls: Vec<String> = Vec::new();
    let mut workers: usize = num_cpus::get(); // Default to number of logical CPU cores
    let mut timeout: u64 = 5; // Default timeout in seconds
    let mut retries: u32 = 0; // Default retries

    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--file" => {
                if i + 1 < args.len() {
                    file_path = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    eprintln!("Error: --file requires a file path");
                    std::process::exit(2);
                }
            }
            "--workers" => {
                if i + 1 < args.len() {
                    workers = args[i + 1].parse().unwrap_or_else(|_| {
                        eprintln!("Error: --workers requires a valid number");
                        std::process::exit(2);
                    });
                    i += 1;
                } else {
                    eprintln!("Error: --workers requires a value");
                    std::process::exit(2);
                }
            }
            "--timeout" => {
                if i + 1 < args.len() {
                    timeout = args[i + 1].parse().unwrap_or_else(|_| {
                        eprintln!("Error: --timeout requires a valid number");
                        std::process::exit(2);
                    });
                    i += 1;
                } else {
                    eprintln!("Error: --timeout requires a value");
                    std::process::exit(2);
                }
            }
            "--retries" => {
                if i + 1 < args.len() {
                    retries = args[i + 1].parse().unwrap_or_else(|_| {
                        eprintln!("Error: --retries requires a valid number");
                        std::process::exit(2);
                    });
                    i += 1;
                } else {
                    eprintln!("Error: --retries requires a value");
                    std::process::exit(2);
                }
            }
            _ => {
                // Treat as a URL
                urls.push(args[i].clone());
            }
        }
        i += 1;
    }

    // Read URLs from file if provided
    if let Some(path) = file_path {
        match fs::read_to_string(&path) {
            Ok(contents) => {
                for line in contents.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        urls.push(line.to_string());
                    }
                }
            }
            Err(err) => {
                eprintln!("Error reading file {}: {}", path, err);
                std::process::exit(2);
            }
        }
    }

    // If no URLs are provided, print usage and exit
    if urls.is_empty() {
        eprintln!("Error: No URLs provided");
        print_usage();
        std::process::exit(2);
    }

    // Create a channel for sending URLs to worker threads
    let (tx, rx) = mpsc::channel::<String>();
    let rx = Arc::new(Mutex::new(rx));

    // Shared vector to collect results
    let results = Arc::new(Mutex::new(Vec::new()));

    // Spawn worker threads
    let mut handles = Vec::new();
    for _ in 0..workers {
        let rx = Arc::clone(&rx);
        let results = Arc::clone(&results);
        let client = Client::new();
        let handle = thread::spawn(move || {
            while let Ok(url) = rx.lock().unwrap().recv() {
                let start = Instant::now();
                let result = check_website(&client, &url, timeout, retries);
                let duration = start.elapsed();

                let status = WebsiteStatus {
                    url: url.clone(),
                    status: result.map_err(|e| e.to_string()),
                    response_time_ms: duration.as_millis(),
                    timestamp: chrono::Local::now().to_rfc3339(),
                };

                // Live output to stdout
                match &status.status {
                    Ok(code) => println!(
                        "[SUCCESS] {} - HTTP {} in {} ms at {}",
                        status.url, code, status.response_time_ms, status.timestamp
                    ),
                    Err(err) => println!(
                        "[FAILURE] {} - {} in {} ms at {}",
                        status.url, err, status.response_time_ms, status.timestamp
                    ),
                }

                // Add the result to the shared vector
                results.lock().unwrap().push(status);
            }
        });
        handles.push(handle);
    }

    // Send URLs to the channel
    for url in urls {
        tx.send(url).expect("Failed to send URL to worker thread");
    }

    // Drop the sender to close the channel
    drop(tx);

    // Wait for all threads to finish
    for handle in handles {
        handle.join().expect("Failed to join worker thread");
    }

    // Calculate summary statistics for successful responses
    let results = results.lock().unwrap();
    let mut times: Vec<u128> = results
        .iter()
        .filter_map(|s| if s.status.is_ok() { Some(s.response_time_ms) } else { None })
        .collect();

    if !times.is_empty() {
        times.sort();
        let min = times.first().unwrap();
        let max = times.last().unwrap();
        let avg = times.iter().sum::<u128>() as f64 / times.len() as f64;
        println!(
            "\nSummary statistics for successful responses:\n  Min: {} ms\n  Max: {} ms\n  Avg: {:.2} ms\n",
            min, max, avg
        );
    } else {
        println!("\nNo successful responses to summarize.\n");
    }

    // Write results to a JSON file manually
    let mut json = String::from("[\n");
    for (i, status) in results.iter().enumerate() {
        let status_str = match &status.status {
            Ok(code) => format!("\"Ok\": {}", code),
            Err(err) => format!("\"Err\": \"{}\"", err),
        };
        let entry = format!(
            "  {{\n    \"url\": \"{}\",\n    \"status\": {{ {} }},\n    \"response_time_ms\": {},\n    \"timestamp\": \"{}\"\n  }}",
            status.url, status_str, status.response_time_ms, status.timestamp
        );
        json.push_str(&entry);
        if i < results.len() - 1 {
            json.push_str(",\n");
        }
    }
    json.push_str("\n]\n");

    let mut file = File::create("status.json").expect("Failed to create status.json");
    file.write_all(json.as_bytes())
        .expect("Failed to write to status.json");

    println!("Results written to status.json");
}

fn check_website(client: &Client, url: &str, timeout: u64, retries: u32) -> Result<u16, String> {
    let mut attempts = 0;

    loop {
        let response = client
            .get(url)
            .timeout(Duration::from_secs(timeout))
            .send();

        match response {
            Ok(resp) => return Ok(resp.status().as_u16()),
            Err(err) => {
                attempts += 1;
                if attempts > retries {
                    return Err(err.to_string());
                }
                // Wait 100ms before retrying
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

fn print_usage() {
    println!("Usage: website_checker [--file sites.txt] [URL ...]");
    println!("               [--workers N] [--timeout S] [--retries N]");
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::blocking::Client;

    #[test]
    fn test_check_website_success() {
        let client = Client::new();
        let url = "https://www.rust-lang.org";
        let result = check_website(&client, url, 5, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_website_failure() {
        let client = Client::new();
        let url = "https://wikipedi@.org";
        let result = check_website(&client, url, 5, 0);
        assert!(result.is_err());
    }
}