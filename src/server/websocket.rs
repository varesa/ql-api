use tokio::net::{TcpListener, TcpStream};
use crate::errors::ApplicationError;
use super::formats;
use futures::{StreamExt, SinkExt};
use tokio_tungstenite::tungstenite::{
    Message,
    handshake::server::{
        Callback, Response, Request, ErrorResponse
    }
};
use std::sync::{Mutex, Arc};
use crate::hub::ClientState;

const BIND_TO: &str = "127.0.0.1:8083";

pub async fn listen(client_state: Arc<Mutex<ClientState>>) -> Result<(), ApplicationError>{
    let listener = TcpListener::bind(BIND_TO).await?;
    println!("Websocket server listening on {}", BIND_TO);

    while let Ok((stream, address)) = listener.accept().await {
        println!("Connection from: {}", address);
        tokio::spawn(handle_client(stream, client_state.clone()));
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
                    "ql-raw" | "ql-json1" => {
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

async fn handle_client(stream: TcpStream, client_state: Arc<Mutex<ClientState>>) {
    let protocol = Arc::new(Mutex::new(String::new()));
    let protocol_filter = ProtocolFilter { protocol: protocol.clone() };

    let ws_stream = tokio_tungstenite::accept_hdr_async(stream, protocol_filter).await
        .expect("Error during the websocket handshake");
    let protocol = protocol.lock().unwrap().clone();
    println!("Protocol: {}", protocol);

    let format: Box<dyn formats::TransportFormat> = match protocol.as_str() {
        "ql-raw" => Box::new(formats::Raw {}),
        "ql-json1" => Box::new(formats::Json1 {}),
        _ => unimplemented!("Protocol {} not implemented", protocol),
    };

    let (mut hub_tx, mut hub_rx) = {
        let mut client_state = client_state.lock().unwrap();
        client_state.add_client()
    };
    let (mut ws_tx, mut ws_rx) = ws_stream.split();

    loop {
        tokio::select! {
            Some(Ok(message_ws)) = ws_rx.next() => {
                let message_client = message_ws.into_text().expect("Failed to convert to text");
                println!("WS R: {}", &message_client);
                let message_ql = format.to_ql(message_client);
                hub_tx.send(message_ql).await.expect("Failed to send");
            }
            Some(message_ql) = hub_rx.next() => {
                let message_client = format.from_ql(message_ql);
                println!("WS S: {}", &message_client);
                ws_tx.send(Message::from(message_client)).await.expect("Failed to send");
            }
        }
    }
}