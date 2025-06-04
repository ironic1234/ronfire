use std::env;
use std::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Remove existing socket file if it exists
    let socket_path = &env::args().collect::<Vec<String>>()[1];
    if std::path::Path::new(socket_path).exists() {
        std::fs::remove_file(socket_path)?;
    }

    let listener = UnixListener::bind(socket_path)?;

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

fn parse_request(request: &str) -> Option<String> {
    let mut lines = request.lines();
    if let Some(first_line) = lines.next() {
        let mut parts = first_line.split_whitespace();
        let method = parts.next().unwrap_or("");
        let path = parts.next().unwrap_or("/");
        let version = parts.next().unwrap_or("");

        if version != "HTTP/1.1" && version != "HTTP/1.0" {
            eprintln!("Unsupported HTTP version: {}", version);
            return None;
        }

        if method == "GET" {
            let path = path.trim_start_matches('/');
            let full_path = if path.is_empty() {
                "static/index.html".to_string()
            } else {
                format!("static/{}", path)
            };

            return Some(full_path);
        } else {
            eprintln!("Unsupported HTTP method: {}", method);
            return None;
        }
    }

    None
}

fn generate_response(full_path: &str) -> String {
    match fs::read_to_string(&full_path) {
        Ok(contents) => format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
            contents.len(),
            contents
        ),
        Err(_) => {
            let body = "<h1>404 Not Found</h1>";
            format!(
                "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
                body.len(),
                body
            )
        }
    }
}
