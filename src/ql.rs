use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_util::codec::{LinesCodec, Framed};

use crate::errors::ApplicationError;
use crate::bidirectional_channel::{bidirectional_channel, ChannelEndpoint};
use futures::{StreamExt, SinkExt};

pub struct QL {
    stream: Framed<TcpStream, LinesCodec>,
    hub_channel: ChannelEndpoint<String>,
}

impl QL {
    pub async fn connect(addr: &SocketAddr) -> Result<(QL, ChannelEndpoint<String>), ApplicationError> {
        let connection = TcpStream::connect(addr).await?;
        let stream = Framed::new(connection, LinesCodec::new());
        println!("Connected");

        let (ql_endpoint, hub_endpoint) = bidirectional_channel();

        let ql = QL {
            stream,
            hub_channel: ql_endpoint,
        };

        Ok((ql, hub_endpoint))
    }

    pub async fn process(&mut self) -> Result<(), ApplicationError> {
        loop {
            tokio::select! {
                Some(Ok(msg)) = self.stream.next() => {
                    println!("R: {:?}", msg);
                    self.hub_channel.tx.send(msg).await?;
                }
                Some(msg) = self.hub_channel.rx.next() => {
                    println!("S: {:?}", msg);
                    self.stream.send(msg).await?;
                }
            }
        }
    }
}