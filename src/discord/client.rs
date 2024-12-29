use crate::discord::message::{
    read_discord_reply, DiscordEmbed, DiscordError, DiscordMessage, MessageBuilder, Reply,
};
use crate::https::https_client::Client;
use crate::https::request::{HTTPMethods, RequestBuilder};
use crate::https::response::Response;
use std::str;

pub struct DiscordClient {
    conn: Client,
    token: String,
    base_headers: Vec<String>,
}

impl DiscordClient {
    pub fn new(token: &str) -> Self {
        let mut base_headers = vec![
            "User-Agent: DiscordBot (none, 0.0.1) Bigeon".to_string(),
            "Content-Type: application/json".to_string(),
        ];
        let auth = format!("Authorization: Bot {}", token);
        base_headers.push(auth);

        Self {
            base_headers: base_headers.to_owned(),
            token: token.to_string(),
            conn: Client::new("discord.com").unwrap(),
        }
    }
    pub fn send_message(
        &mut self,
        msg: DiscordMessage,
        channel_id: &str,
    ) -> Result<Box<dyn Reply>, Box<dyn std::error::Error>> {
        let mut buf: [u8; 4096] = [0; 4096];
        let req = RequestBuilder::new()
            .set_method(HTTPMethods::POST)
            .set_host(&self.conn.server_name)
            .add_many_headers(&self.base_headers)
            .set_route(&format!("/api/v10/channels/{}/messages", channel_id))
            .set_content(&msg.to_vec()?)
            .build();

        self.conn.client_write(&req)?;
        let len = self.conn.client_read(&mut buf)?;
        let http_response = Response::from_bytes(&buf[0..len])?;
        let discord_response = str::from_utf8(&http_response.content)?;
        read_discord_reply(discord_response)
    }
}
