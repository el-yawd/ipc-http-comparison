use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
struct PingMessage {
    message: String,
    timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct PongMessage {
    message: String,
    original_timestamp: u64,
    response_timestamp: u64,
}

#[derive(Parser)]
#[command(name = "client")]
#[command(about = "A client to test HTTP vs IPC latency")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Http {
        #[arg(short, long, default_value = "http://http-service:3000")]
        url: String,
        #[arg(short, long, default_value_t = 100)]
        requests: u32,
    },
    Ipc {
        #[arg(short, long, default_value = "/tmp/ipc-service.sock")]
        socket_path: String,
        #[arg(short, long, default_value_t = 100)]
        requests: u32,
    },
    Compare {
        #[arg(short, long, default_value = "http://http-service:3000")]
        url: String,
        #[arg(short, long, default_value = "/tmp/ipc-service.sock")]
        socket_path: String,
        #[arg(short, long, default_value_t = 100)]
        requests: u32,
    },
}

async fn test_http(url: &str, requests: u32) -> anyhow::Result<Vec<Duration>> {
    let client = reqwest::Client::new();
    let mut latencies = Vec::new();

    println!("Testing HTTP with {} requests...", requests);

    for i in 0..requests {
        let ping = PingMessage {
            message: format!("Hello {}", i),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        };

        let start = Instant::now();
        let response = client
            .post(&format!("{}/ping", url))
            .json(&ping)
            .send()
            .await?;

        let _pong: PongMessage = response.json().await?;
        let latency = start.elapsed();
        latencies.push(latency);

        if i % 10 == 0 {
            println!("Completed {} requests", i + 1);
        }
    }

    Ok(latencies)
}

fn test_ipc(socket_path: &str, requests: u32) -> anyhow::Result<Vec<Duration>> {
    let mut stream = UnixStream::connect(socket_path)?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut latencies = Vec::new();

    println!("Testing IPC with {} requests...", requests);

    for i in 0..requests {
        let ping = PingMessage {
            message: format!("Hello {}", i),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        };

        let ping_json = serde_json::to_string(&ping)?;

        let start = Instant::now();
        writeln!(stream, "{}", ping_json)?;
        stream.flush()?;

        let mut response_line = String::new();
        reader.read_line(&mut response_line)?;
        let _pong: PongMessage = serde_json::from_str(response_line.trim())?;
        let latency = start.elapsed();
        latencies.push(latency);

        if i % 10 == 0 {
            println!("Completed {} requests", i + 1);
        }
    }

    Ok(latencies)
}

fn print_stats(name: &str, latencies: &[Duration]) {
    if latencies.is_empty() {
        println!("{}: No data", name);
        return;
    }

    let mut sorted_latencies = latencies.to_vec();
    sorted_latencies.sort();

    let min = sorted_latencies[0];
    let max = sorted_latencies[sorted_latencies.len() - 1];
    let avg = sorted_latencies.iter().sum::<Duration>() / sorted_latencies.len() as u32;
    let p50 = sorted_latencies[sorted_latencies.len() / 2];
    let p95 = sorted_latencies[(sorted_latencies.len() as f64 * 0.95) as usize];
    let p99 = sorted_latencies[(sorted_latencies.len() as f64 * 0.99) as usize];

    println!("\n{} Results:", name);
    println!("  Min:    {:?}", min);
    println!("  Max:    {:?}", max);
    println!("  Avg:    {:?}", avg);
    println!("  P50:    {:?}", p50);
    println!("  P95:    {:?}", p95);
    println!("  P99:    {:?}", p99);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Http { url, requests } => {
            let latencies = test_http(&url, requests).await?;
            print_stats("HTTP", &latencies);
        }
        Commands::Ipc {
            socket_path,
            requests,
        } => {
            let latencies = test_ipc(&socket_path, requests)?;
            print_stats("IPC", &latencies);
        }
        Commands::Compare {
            url,
            socket_path,
            requests,
        } => {
            println!("Running comparison test...\n");

            // Wait a bit for services to be ready
            tokio::time::sleep(Duration::from_secs(2)).await;

            let http_latencies = test_http(&url, requests).await?;
            print_stats("HTTP", &http_latencies);

            let ipc_latencies = test_ipc(&socket_path, requests)?;
            print_stats("IPC", &ipc_latencies);

            // Calculate improvement
            if !http_latencies.is_empty() && !ipc_latencies.is_empty() {
                let http_avg =
                    http_latencies.iter().sum::<Duration>() / http_latencies.len() as u32;
                let ipc_avg = ipc_latencies.iter().sum::<Duration>() / ipc_latencies.len() as u32;

                let improvement = if http_avg > ipc_avg {
                    format!(
                        "IPC is {:.2}x faster",
                        http_avg.as_nanos() as f64 / ipc_avg.as_nanos() as f64
                    )
                } else {
                    format!(
                        "HTTP is {:.2}x faster",
                        ipc_avg.as_nanos() as f64 / http_avg.as_nanos() as f64
                    )
                };

                println!("\nComparison: {}", improvement);
            }
        }
    }

    Ok(())
}
