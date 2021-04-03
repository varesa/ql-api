use tokio::net::{TcpListener, TcpStream};
use crate::errors::ApplicationError;
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

async fn handle_client(stream: TcpStream, client_state: Arc<Mutex<ClientState>>) {
    let protocol = Arc::new(Mutex::new(String::new()));
    let protocol_filter = ProtocolFilter { protocol: protocol.clone() };

    let ws_stream = tokio_tungstenite::accept_hdr_async(stream, protocol_filter).await
        .expect("Error during the websocket handshake");
    println!("Protocol: {}", protocol.lock().unwrap());

    let (mut hub_tx, mut hub_rx) = {
        let mut client_state = client_state.lock().unwrap();
        client_state.add_client()
    };
    let (mut ws_tx, mut ws_rx) = ws_stream.split();

    loop {
        tokio::select! {
            Some(Ok(message)) = ws_rx.next() => {
                //println!("x: {:?}", x.into_text());
                hub_tx.send(message.into_text().expect("Failed to convert to text")).await.expect("Failed to send");
            }
            Some(message) = hub_rx.next() => {
                println!("ws: {}", &message);
                let x = ws_tx.send(Message::from(message)).await; //.expect("Failed to send");
                println!("x: {:?}", x);
            }
        }
    }
}