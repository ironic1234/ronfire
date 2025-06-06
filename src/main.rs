use std::env;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use ronaks_webserver::{create_socket, parse_request, generate_response};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = env::args().nth(1).expect("Missing socket path");

    let listener = create_socket(socket_path).expect("Could not create socket");

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            match socket.read(&mut buf).await {
                Ok(n) if n == 0 => return, // closed
                Ok(n) => {
                    let request = String::from_utf8_lossy(&buf[..n]);
                    if let Some(full_path) = parse_request(&request) {
                        let response = generate_response(&full_path);
                        if let Err(e) = socket.write_all(response.as_bytes()).await {
                            eprintln!("Failed to write to socket: {:?}", e);
                        }
                    } else {
                        eprintln!("Invalid request: {}", request);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read from socket: {:?}", e);
                }
            }
        });
    }
}


