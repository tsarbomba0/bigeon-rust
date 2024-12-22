use crate::discord::message::{
    read_discord_reply, DiscordEmbed, DiscordError, DiscordMessage, MessageBuilder, Reply,
};
use crate::https::https_client::Client;
use crate::https::request::{HTTPMethods, Request, RequestBuilder};
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

        base_headers.push(format!("Authorization: Bot {}", token));

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
        let mut buf = Vec::new();
        let request = RequestBuilder::new()
            .set_method(HTTPMethods::POST)
            .set_route(&format!("/api/v10/channels/{}/messages", channel_id))
            .set_host("discord.com")
            .add_many_headers(&self.base_headers)
            .set_content(msg.to_str().unwrap())
            .build()
            .process()
            .unwrap();

        self.conn.client_write(&request).unwrap();
        let l = self.conn.client_read(&mut buf).unwrap();
        println!("Read: {}", l);
        let a = read_discord_reply(str::from_utf8(&buf).unwrap());
        a
    }
}
