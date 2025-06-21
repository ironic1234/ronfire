use ronfire::{
    create_socket, generate_response, parse_request, read_socket, send_response,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/ronfire.sock".to_string());

    let listener = create_socket(socket_path).expect("Could not create socket");

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            loop {
                match read_socket(&mut socket).await {
                    Ok(request) => {
                        // Check for keep-alive
                        let keep_alive = request
                            .contains("Connection: keep-alive")
                            || (request.contains("HTTP/1.1")
                                && !request.contains("Connection: close"));

                        if let Some(full_path) = parse_request(&request) {
                            let mut response = generate_response(&full_path);

                            // Append appropriate Connection header
                            let connection_header = if keep_alive {
                                "Connection: keep-alive\r\n"
                            } else {
                                "Connection: close\r\n"
                            };

                            // Insert Connection header into response
                            response.1 =
                                format!("{}{}", connection_header, response.1);

                            send_response(&mut socket, response).await;
                        } else {
                            eprintln!("Invalid request: {}", request);
                            break;
                        }

                        if !keep_alive {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read from socket: {:?}", e);
                        break;
                    }
                }
            }
        });
    }
}
