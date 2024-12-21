use crate::discord::message::{DiscordEmbed, DiscordError, DiscordMessage, MessageBuilder, Reply};
use crate::https::https_client::Client;
use crate::https::parse_http::parse_http;
use crate::https::request::{Request, RequestBuilder};
use crate::https::response::Response;

pub struct DiscordClient<'a> {
    conn: Client,
    token: String,
    base_headers: Vec<&'a str>,
}

impl<'a> DiscordClient<'a> {
    pub fn new(token: &str) -> Self {
        let base_headers = vec!["User-Agent: DiscordBot (none, 0.0.1) Bigeon",
                                "Content-Type: application/json"];

        base_headers.push("Authorization: Bot "+token)

        Self {
            token: token.to_owned(),
            base_headers,
            conn: Client::new("https://discord.com").unwrap(),
        }
    }
    pub async fn send_message(&mut self, msg: DiscordMessage) -> Result<Box<dyn Reply>, Box<dyn std::error::Error>> {
            let request = RequestBuilder::new()
            .add_many_headers(self.base_headers)
            .build();
            let result = self.conn.client_write(&request.process()?).await?;

            let mut buf = vec![];
            
            let read = self.conn.client_read(&mut buf)?;
            Ok(buf)
        }
}
