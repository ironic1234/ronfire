use tokio::net::UnixListener;
use std::fs;

pub fn create_socket(socket_path: String) -> std::io::Result<UnixListener> {
    if std::path::Path::new(&socket_path).exists() {
        let _ = std::fs::remove_file(&socket_path);
    }

    let listener = UnixListener::bind(socket_path)?;
    Ok(listener)
}

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
