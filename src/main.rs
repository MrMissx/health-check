use std::env;
use std::io::{ErrorKind, Read, Result, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;
use std::time::Instant;

const HOST: &str = "localhost:8219";

fn authenticate(mut stream: &TcpStream, auth_token: &String) -> bool {
    let mut buff = [0; 1024];

    match stream.read(&mut buff) {
        Ok(size) => {
            let token = std::str::from_utf8(&buff[..size]).unwrap_or("");
            if token.trim() != auth_token {
                println!("Authentication failed! Closing connection...");
                stream
                    .write_all(b"Invalid authentication token!\n")
                    .expect("Failed to write to stream");
                return false;
            }
            stream
                .write_all(b"Authenticated\n")
                .expect("Failed to write to stream");
            return true;
        }
        Err(_) => {
            println!("Error reading authentication token");
            return false;
        }
    }
}

fn handle_stream(mut stream: TcpStream) {
    let mut buff = [0; 1024];
    let auth_token = env::var("AUTH_TOKEN").unwrap_or("TOKEN".to_string());
    println!("Authenticating with token: {}", auth_token);

    if !authenticate(&stream, &auth_token) {
        match stream.shutdown(Shutdown::Both) {
            Ok(()) => println!("Connection successfully shut down."),
            Err(e) => match e.kind() {
                ErrorKind::NotConnected => {
                    // If the stream is not connected, log it but don't panic
                    println!("Attempted to shut down a not connected stream.");
                }
                _ => println!("An error occurred while shutting down the stream: {:?}", e),
            },
        }
        return;
    }

    loop {
        match stream.read(&mut buff) {
            Ok(0) => {
                println!("Client {} closed connection", stream.peer_addr().unwrap());
                return;
            }

            Ok(size) => {
                let start = Instant::now();
                let raw = String::from_utf8_lossy(&buff[..size]);
                let incoming = raw.trim();

                println!("Received: {}", incoming);
                let result = match incoming {
                    "ping" => format!("pong! {:?}\n", start.elapsed()),
                    _ => "unknown command\n".to_string(),
                };

                stream
                    .write_all(result.as_bytes())
                    .expect("Failed to write to stream");
            }
            Err(e) => {
                eprintln!("An error occurred while reading from the stream: {:?}", e);
            }
        }
    }
}

fn main() -> Result<()> {
    let listener = TcpListener::bind(HOST).unwrap();
    println!("Server running on {}", HOST);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Connection established with {}!", stream.peer_addr()?);
                thread::spawn(|| handle_stream(stream));
            }
            Err(e) => {
                println!("Failed to establish a connection: {}", e);
            }
        }
    }

    Ok(())
}
