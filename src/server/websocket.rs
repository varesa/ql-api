use tokio::net::{TcpListener, TcpStream};
use crate::errors::ApplicationError;
use futures::StreamExt;
use tokio_tungstenite::tungstenite::handshake::server::{Callback, Response, Request, ErrorResponse};
use std::sync::{Mutex, Arc};

const BIND_TO: &str = "127.0.0.1:8083";

pub async fn test() -> Result<(), ApplicationError>{
    let listener = TcpListener::bind(BIND_TO).await?;
    println!("Websocket server listening on {}", BIND_TO);

    while let Ok((stream, address)) = listener.accept().await {
        println!("Connection from: {}", address);
        tokio::spawn(handle_client(stream));
    }

    Ok(())
}

#[derive(Default, Debug)]
struct ProtocolFilter {
    protocol: Arc<Mutex<String>>,
}

impl Callback for ProtocolFilter {
    fn on_request(self, request: &Request, response: Response) -> Result<Response, ErrorResponse> {
        match request.headers().get("sec-websocket-protocol") {
            None => {
                Err(ErrorResponse::new(Some("Websocket subprotocol missing".into())))
            }
            Some(protocol) => {
                let protocol = protocol.to_str().expect("Failed to convert header to String".into());
                match protocol {
                    "ql-raw" => {
                        self.protocol.lock().unwrap().insert_str(0, protocol);
                        Ok(response)
                    }
                    _ => {
                        Err(ErrorResponse::new(Some("Unsupported protocol".into())))
                    }
                }
            }
        }
    }
}

async fn handle_client(stream: TcpStream) {
    let protocol = Arc::new(Mutex::new(String::new()));
    let protocol_filter = ProtocolFilter { protocol: protocol.clone() };

    let ws_stream = tokio_tungstenite::accept_hdr_async(stream, protocol_filter).await
        .expect("Error during the websocket handshake");
    let (write, read) = ws_stream.split();
    println!("Protocol: {}", protocol.lock().unwrap());
    read.forward(write).await.expect("Failed to forward message");
}