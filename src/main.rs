mod errors;
mod ql;
mod hub;
mod bidirectional_channel;

use std::env;
use std::net::{ToSocketAddrs, SocketAddr};
use crate::hub::Hub;
use crate::ql::QL;
use crate::errors::ApplicationError::{self, UsageError, NameResolutionError};

fn get_address(args: Vec<String>) -> Result<SocketAddr, ApplicationError> {
    if args.len() < 2 || args.len() > 3  {
        return Err(UsageError(format!("{} <address> [<port>]", &args[0])).into());
    }

    let host = args[1].clone();
    let port = if args.len() == 3 { args[2].clone() } else { "49280".into() };

    let address_string = format!("{}:{}", host, port);
    address_string.to_socket_addrs()?.next()
        .ok_or(NameResolutionError(format!("No addresses returned for {}", address_string)))
}

#[tokio::main]
async fn main() -> Result<(), ApplicationError>{
    let args: Vec<String> = env::args().collect();
    let address = get_address(args)?;

    println!("Connecting to {:?}", &address);

    let mut hub = Hub::new().await;

    let ql_task = tokio::task::spawn(async move {
        QL::new(&address, &mut hub).await?.process().await?;
        Result::<(), ApplicationError>::Ok(())
    });

    let handles = vec![ql_task];
    for result in futures::future::join_all(handles).await {
        result??;
    }


    return Ok(());
}
