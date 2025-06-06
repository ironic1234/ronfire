# ronaks_webserver

`ronaks_webserver` is a minimal asynchronous static file server built using Rust and Tokio, communicating over a Unix domain socket. It serves files from a local `static/` directory using a simplified HTTP protocol.

## Features

- Asynchronous I/O via `tokio`
- Uses a Unix domain socket (not TCP)
- Serves static files (HTML) from a `static/` directory
- Basic `GET` request parsing
- Returns `404 Not Found` for missing files
- `Mime-Type` guessing
- Sanitizes path traversal attempts

## Requirements

- Rust (edition 2021)
- Unix-like OS (Linux/macOS)
- Tokio runtime (`[dependencies] tokio = { version = "1", features = ["full"] }`)

## Setup

### 1. Create a `static/` directory

```bash
mkdir static
echo "<h1>Hello, world!</h1>" > static/index.html
````

### 2. Build and run the server

```bash
cargo run -- /tmp/ronak.sock
```

This will:

* Remove any existing socket file at `/tmp/ronak.sock`
* Start listening for HTTP-like requests over the socket

### 3. Test the server

Use `curl` or similar tools:

```bash
curl --unix-socket /tmp/ronak.sock http://localhost/
```

Expected output:

```html
<h1>Hello, world!</h1>
```

## Example Request / Response

### Request

```
GET /index.html HTTP/1.1
Host: localhost
```

### Response

```
HTTP/1.1 200 OK
Content-Length: 27
Content-Type: text/html

<h1>Hello, world!</h1>
```

If the file is missing:

```
HTTP/1.1 404 Not Found
Content-Length: 22
Content-Type: text/html

<h1>404 Not Found</h1>
```

## Caveats

* Only handles basic `GET` requests.

## License

MIT
