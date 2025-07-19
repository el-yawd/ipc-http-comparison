use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;

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

fn handle_client(mut stream: UnixStream) -> anyhow::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                match serde_json::from_str::<PingMessage>(line) {
                    Ok(ping) => {
                        let response = PongMessage {
                            message: format!("Pong! Received: {}", ping.message),
                            original_timestamp: ping.timestamp,
                            response_timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_nanos() as u64,
                        };

                        let response_json = serde_json::to_string(&response)?;
                        writeln!(stream, "{}", response_json)?;
                        stream.flush()?;
                    }
                    Err(e) => {
                        eprintln!("Failed to parse message: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from client: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let socket_path = "/tmp/ipc-service.sock";

    // Remove the socket file if it exists
    if Path::new(socket_path).exists() {
        std::fs::remove_file(socket_path)?;
    }

    let listener = UnixListener::bind(socket_path)?;
    println!("IPC Service listening on {}", socket_path);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    if let Err(e) = handle_client(stream) {
                        eprintln!("Error handling client: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }

    Ok(())
}
