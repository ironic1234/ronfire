use ronfire::{
    AsyncLogger, create_socket, generate_response, parse_request, read_socket,
    send_response,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/ronfire.sock".to_string());

    let logger = AsyncLogger::new();
    let listener = create_socket(socket_path).expect("Could not create socket");

    loop {
        let (mut socket, _) = listener.accept().await?;
        let logger = logger.clone();

        tokio::spawn(async move {
            loop {
                match read_socket(&mut socket).await {
                    Ok(request) => {
                        // Check for keep-alive
                        let keep_alive = request
                            .contains("Connection: keep-alive")
                            || (request.contains("HTTP/1.1")
                                && !request.contains("Connection: close"));

                        if let Some(full_path) =
                            parse_request(&request, Some(&logger)).await
                        {
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

                            send_response(&mut socket, response, Some(&logger))
                                .await;
                        } else {
                            logger
                                .log(&format!("Invalid request: {}", request))
                                .await;
                            break;
                        }

                        if !keep_alive {
                            break;
                        }
                    }
                    Err(e) => {
                        logger
                            .log(&format!(
                                "Failed to read from socket: {:?}",
                                e
                            ))
                            .await;
                        break;
                    }
                }
            }
        });
    }
}
