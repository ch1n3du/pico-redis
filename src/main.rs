// Uncomment this block to pass the first stage
use std::net::TcpListener;

use redis_starter_rust::{app::App, error::Error};

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let mut app = App::new().await;
    if let Err(err) = app.run().await {
        match err {
            Error::ConnectionClosed => println!("Peer closed connection unexpectedly."),
            Error::IncompleteRequestData => println!("Request data incomplete"),
            _ => todo!(),
        }
    }
}
