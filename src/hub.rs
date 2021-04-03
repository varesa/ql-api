use crate::bidirectional_channel::ChannelEndpoint;

pub struct Hub {
    ql_channel: Option<ChannelEndpoint<String>>,
}

impl Hub {
    pub async fn new() -> Hub {
        Hub {
            ql_channel: None
        }
    }

    pub async fn register_ql_channel(&mut self, channel_endpoint: ChannelEndpoint<String>) {
        self.ql_channel = Some(channel_endpoint);
    }
}