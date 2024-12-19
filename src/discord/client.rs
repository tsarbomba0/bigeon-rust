use crate::discord::message::{DiscordEmbed, DiscordError, DiscordMessage, MessageBuilder, Reply};
use crate::https::https_client::Client;
use crate::https::parse_http::parse_http;
use crate::https::request::{Request, RequestBuilder};
use crate::https::response::Response;

pub struct DiscordClient {
    conn: Client,
    token: String,
    base_headers: Vec<String>,
}

impl DiscordClient {
    pub fn new(token: &str, headers: Option<&Vec<String>>) -> Self {
        let base_headers = match headers {
            Some(h) => h.to_owned(),
            None => vec![],
        };

        Self {
            token: token.to_owned(),
            base_headers,
            conn: Client::new("https://discord.com").unwrap(),
        }
    }
    pub async fn send_message(&mut self, msg: DiscordMessage) -> Result<Box<dyn Reply>, Box<dyn std::error::Error>> {
            let request = RequestBuilder::new()
            .headers("Authorization:")
            self.conn.client_write()
    }
}
