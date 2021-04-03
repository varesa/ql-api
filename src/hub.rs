use crate::bidirectional_channel::ChannelEndpoint;
use futures::channel::mpsc::{channel, Sender, Receiver};
use futures::{StreamExt, SinkExt};
use std::sync::{Arc, Mutex};
use crate::errors::ApplicationError;

const BUF_SIZE: usize = 1024;

pub struct ClientState {
    // Template to clone the client end of the channel
    pub shared_client_sender: Sender<String>,

    // List of all senders to clients
    clients: Vec<Sender<String>>,
}

impl ClientState {
    pub fn add_client(&mut self) -> (Sender<String>, Receiver<String>) {
        let (hub_sender, client_receiver) = channel(BUF_SIZE);
        self.clients.push(hub_sender);
        (self.shared_client_sender.clone(), client_receiver)
    }
}

pub struct Hub {
    pub client_state: Arc<Mutex<ClientState>>,
    ql_channel: ChannelEndpoint<String>,
    shared_client_receiver: Receiver<String>,
}

impl Hub {
    pub async fn new(channel_endpoint: ChannelEndpoint<String>) -> Hub {
        let (client_sender, client_receiver) = channel(BUF_SIZE);
        Hub {
            shared_client_receiver: client_receiver,
            ql_channel: channel_endpoint,
            client_state: Arc::new(Mutex::new(ClientState {
                clients: Vec::new(),
                shared_client_sender: client_sender,
            })),
        }
    }

    pub async fn process(&mut self) -> Result<(), ApplicationError> {
        loop {
            tokio::select! {
                Some(message) = self.ql_channel.rx.next() => {
                    println!("hub: {}", message);
                    for client in &mut self.client_state.lock().unwrap().clients {
                        let mut client = client.clone();
                        let message = message.clone();
                        tokio::spawn(async move { client.send(message).await });
                    }
                }
                Some(message) = self.shared_client_receiver.next() => { self.ql_channel.tx.send(message).await?; }
            }
        }
    }
}