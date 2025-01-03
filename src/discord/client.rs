use crate::discord::message::{read_discord_reply, DiscordMessage, Reply};
use crate::https::persistent_client::PersistentClient;
use crate::https::response::Response;
use std::collections::HashMap;
use std::io::Error;
use std::str;

const DISCORD_USER_AGENT: &str = "DiscordBot (Bigeon, 0.0.2)";
const DISCORD_API_URL: &str = "https://discord.com/api/v10";

type HeaderMap<'a> = HashMap<&'a str, String>;

pub struct DiscordClient<'a> {
    conn: PersistentClient<'a>,
    token: &'a str,
    headers: HeaderMap<'a>,
}

impl<'a> DiscordClient<'a> {
    pub fn new(token: &'a str) -> Result<Self, Error> {
        let token_string = format!("Bot {}", token);
        let base_headers = vec![
            ("Content-Type", "application/json".to_string()),
            ("Authorization", token_string),
        ]
        .into_iter()
        .collect::<HeaderMap<'a>>();
        let conn = PersistentClient::new(DISCORD_USER_AGENT, "https://discord.com")?;

        Ok(Self {
            token,
            conn,
            headers: base_headers,
        })
    }
    pub fn send_message(
        &mut self,
        msg: DiscordMessage,
        channel_id: &'a str,
    ) -> Result<Box<dyn Reply>, Box<dyn std::error::Error>> {
        let msg_bytes = msg.to_vec()?;
        let url = format!("{}/channels/{}/messages", DISCORD_API_URL, channel_id);
        let reply = match self.conn.post(&url) {
            Ok(mut r) => r.content(&msg_bytes).headers(&self.headers).execute()?,
            Err(e) => panic!("{}", e),
        };
        let resp = Response::from_slice(&reply)?;

        println!("{:#?}", resp);
        let discord_response = str::from_utf8(&resp.content)?;
        read_discord_reply(discord_response)
    }
}
