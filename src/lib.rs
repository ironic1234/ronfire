use mime_guess::from_path;
use std::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixStream, UnixListener};

/// Creates a Unix domain socket at the given path.
///
/// This function removes the socket file if it already exists at the specified path,
/// then binds a new `tokio::net::UnixListener` to that path.
///
/// # Arguments
///
/// * `socket_path` - A `String` representing the filesystem path to the socket file.
///
/// # Returns
///
/// Returns a `Result` containing a bound `UnixListener` on success, or a `std::io::Error` on failure.
pub fn create_socket(socket_path: String) -> std::io::Result<UnixListener> {
    if std::path::Path::new(&socket_path).exists() {
        let _ = std::fs::remove_file(&socket_path);
    }

    let listener = UnixListener::bind(socket_path)?;
    Ok(listener)
}

// Parses a raw HTTP request string and extracts the target file path.
///
/// This function supports only `GET` requests with HTTP/1.0 or HTTP/1.1. It trims the leading `/`
/// from the path and prepends `"static/"` to resolve the file path. If the path is empty, it defaults
/// to `"static/index.html"`.
///
/// # Arguments
///
/// * `request` - A string slice representing the raw HTTP request.
///
/// # Returns
///
/// Returns `Some(String)` with the resolved file path if the request is valid and supported, or `None` otherwise.
pub fn parse_request(request: &str) -> Option<String> {
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

/// Generates a complete HTTP response based on the contents of a file.
///
/// If the file exists and can be read, it returns a `200 OK` response with the file contents.
/// If the file cannot be read, it returns a `404 Not Found` response with a simple error message.
///
/// # Arguments
///
/// * `full_path` - A string slice representing the path to the file to be served.
///
/// # Returns
///
/// A complete HTTP response as a `String`, including status line, headers, and body.
pub fn generate_response(full_path: &str) -> (String, String, Vec<u8>) {
    match fs::read(full_path) {
        Ok(contents) => {
            let mime_type = from_path(full_path)
                .first_or_octet_stream()
                .essence_str()
                .to_string();

            let status_line = "HTTP/1.1 200 OK\r\n".to_string();
            let headers = format!(
                "Content-Length: {}\r\nContent-Type: {}\r\n\r\n",
                contents.len(),
                mime_type
            );

            (status_line, headers, contents)
        }
        Err(_) => {
            let body = b"<h1>404 Not Found</h1>".to_vec();
            let status_line = "HTTP/1.1 404 Not Found\r\n".to_string();
            let headers = format!(
                "Content-Length: {}\r\nContent-Type: text/html\r\n\r\n",
                body.len()
            );

            (status_line, headers, body)
        }
    }
}

/// Sends an HTTP-like response over the provided UnixStream socket asynchronously.
///
/// The response is sent in three parts: status line, headers, and body.
/// Each part is written sequentially to the socket. Errors are logged to stderr,
/// and the function returns early if writing the status or headers fails.
///
/// # Arguments
///
/// * `socket` - The UnixStream to send the response through.
/// * `response_parts` - A tuple containing the status line (String),
///   headers (String), and body (Vec<u8>).
pub async fn send_response(mut socket: UnixStream, response_parts: (String, String, Vec<u8>)) {
    let (status, headers, body) = response_parts;

    if let Err(e) = socket.write_all(status.as_bytes()).await {
        eprintln!("Failed to write status line: {}", e);
        return;
    }

    if let Err(e) = socket.write_all(headers.as_bytes()).await {
        eprintln!("Failed to write headers: {}", e);
        return;
    }

    if let Err(e) = socket.write_all(&body).await {
        eprintln!("Failed to write body: {}", e);
    }
}

/// Reads data from the provided UnixStream socket asynchronously.
/// 
/// Returns a tuple containing the request as a String and the socket itself.
/// 
/// # Errors
/// Returns an error if reading from the socket fails.
pub async fn read_socket(mut socket: UnixStream) -> Result<(String, UnixStream), std::io::Error> {
    let mut buf = [0; 1024];
    let n = socket.read(&mut buf).await?;
    Ok((String::from_utf8_lossy(&buf[..n]).to_string(), socket))
}
