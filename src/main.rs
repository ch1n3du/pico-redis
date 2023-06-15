// Uncomment this block to pass the first stage
use std::net::TcpListener;

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    loop {
        let incoming = listener.accept();
        match incoming {
            Ok((_stream, addr)) => {
                println!("Accepted new connection from {addr}");
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
