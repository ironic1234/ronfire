use std::path::{Path, PathBuf};
use std::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

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

            // Reject paths that try to traverse outside the static directory
            if path.split('/').any(|part| part == "..") {
                eprintln!("Path traversal attempt: {}", path);
                return None;
            }

            return resolve_static_path(path);
        } else {
            eprintln!("Unsupported HTTP method: {}", method);
            return None;
        }
    }

    None
}

/// Resolves a user-facing URL path to a static file path on disk using fallback rules.
///
/// This function applies simple fallback logic for friendly URLs:
///
/// - An empty path (e.g., `/`) maps to `static/index.html`.
/// - A path ending with `/` (e.g., `/blog/`) maps to `static/blog/index.html`.
/// - A path without an extension (e.g., `/about`) tries:
///     - `static/about.html`
///     - `static/about/index.html`
/// - A path with an extension (e.g., `/style.css`) is used as-is.
///
/// The first path that exists and is a regular file is returned.
///
/// # Arguments
///
/// * `path` - A normalized URL path without a leading slash (e.g., `about`, `blog/`, `css/main.css`).
///
/// # Returns
///
/// * `Some(String)` if a valid file is found under the `static/` directory.
/// * `None` if no matching file exists.
fn resolve_static_path(path: &str) -> Option<String> {
    let static_dir = PathBuf::from("static");

    let candidates = if path.is_empty() {
        vec![static_dir.join("index.html")]
    } else if path.ends_with('/') {
        vec![static_dir.join(path).join("index.html")]
    } else if Path::new(path).extension().is_none() {
        vec![
            static_dir.join(format!("{path}.html")),
            static_dir.join(path).join("index.html"),
        ]
    } else {
        vec![static_dir.join(path)]
    };

    for candidate in candidates {
        if candidate.exists() && candidate.is_file() {
            return Some(candidate.to_string_lossy().to_string());
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
            let mime_type = guess_mime_type(full_path);

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

/// Guesses the MIME type of a file based on its extension.
///
/// This function inspects the file extension of the provided path string and returns
/// a corresponding MIME type string. It handles a variety of common web and media formats.
///
/// # Arguments
///
/// * `path` - A string slice representing the file path. Only the file extension is used.
///
/// # Returns
///
/// A string slice representing the MIME type. If the extension is not recognized,
/// `"application/octet-stream"` is returned as a default fallback.
///
/// # Examples
///
/// ```
/// let mime = guess_mime_type("index.html");
/// assert_eq!(mime, "text/html");
///
/// let mime = guess_mime_type("image.jpeg");
/// assert_eq!(mime, "image/jpeg");
///
/// let mime = guess_mime_type("unknownfile.xyz");
/// assert_eq!(mime, "application/octet-stream");
/// ```
fn guess_mime_type(path: &str) -> &'static str {
    match Path::new(path).extension().and_then(|s| s.to_str()) {
        Some("html") => "text/html",
        Some("htm") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("txt") => "text/plain",
        Some("wasm") => "application/wasm",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("ogg") => "audio/ogg",
        Some("mp3") => "audio/mpeg",
        _ => "application/octet-stream", // default fallback
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
///   headers (String), and body (`Vec<u8>`).
pub async fn send_response(
    socket: &mut UnixStream,
    response_parts: (String, String, Vec<u8>),
) {
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
pub async fn read_socket(
    socket: &mut UnixStream,
) -> Result<String, std::io::Error> {
    let mut buf = [0; 1024];
    let n = socket.read(&mut buf).await?;
    Ok(String::from_utf8_lossy(&buf[..n]).to_string())
}


