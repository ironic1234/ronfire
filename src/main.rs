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
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            match read_socket(socket).await {
                Ok((request, socket)) => {
                    if let Some(full_path) = parse_request(&request) {
                        let response = generate_response(&full_path);
                        send_response(socket, response).await;
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
