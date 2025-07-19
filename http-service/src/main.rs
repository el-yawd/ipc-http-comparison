use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use warp::Filter;

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

async fn ping_handler(ping: PingMessage) -> Result<impl warp::Reply, Infallible> {
    let response = PongMessage {
        message: format!("Pong! Received: {}", ping.message),
        original_timestamp: ping.timestamp,
        response_timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64,
    };

    Ok(warp::reply::json(&response))
}

#[tokio::main]
async fn main() {
    // POST /ping
    let ping_route = warp::path("ping")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(ping_handler);

    // Health check
    let health_route = warp::path("health")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({"status": "ok"})));

    let routes = ping_route.or(health_route);

    println!("HTTP Service starting on port 3000...");
    warp::serve(routes).run(([0, 0, 0, 0], 3000)).await;
}
