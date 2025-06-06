use tokio::net::UnixListener;
use std::fs;

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
pub fn generate_response(full_path: &str) -> String {
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
