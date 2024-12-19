use serde::{Deserialize, Serialize};
use serde_json;

pub struct DiscordEmbed {}

#[derive(Debug, Deserialize, Serialize)]
pub struct DiscordError {
    code: u32,
    errors: Vec<String>, // todo
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DiscordMessage {
    content: Option<String>,
    nonce: Option<String>,
    tts: Option<bool>,
    embeds: Option<Vec<String>>,
    allowed_mentions: Option<Vec<String>>, // TODO
    message_reference: Option<String>,     // Message reference,
    components: Option<Vec<String>>,       // TODO
    sticker_ids: Option<Vec<String>>,
    attachments: Option<Vec<String>>, // TODO
    flags: Option<u32>,
    enforce_nonce: Option<bool>,
    poll: Option<String>, // TODO
}

// Message Builder
pub struct MessageBuilder {
    content: Option<String>,
    nonce: Option<String>,
    tts: Option<bool>,
    embeds: Option<Vec<String>>,
    allowed_mentions: Option<Vec<String>>, // TODO
    message_reference: Option<String>,     // Message reference,
    components: Option<Vec<String>>,       // TODO
    sticker_ids: Option<Vec<String>>,
    attachments: Option<Vec<String>>, // TODO
    flags: Option<u32>,
    enforce_nonce: Option<bool>,
    poll: Option<String>, // TODO
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self {
            content: None,
            nonce: None,
            tts: None,
            embeds: None,
            allowed_mentions: None,
            message_reference: None,
            components: None,
            sticker_ids: None,
            attachments: None,
            flags: None,
            enforce_nonce: None,
            poll: None,
        }
    }

    pub fn content(&mut self, c: &str) -> &mut Self {
        self.content = Some(c.to_owned());
        self
    }
}

// Enum
enum DiscordReply {
    DiscordMessage,
    DiscordError,
}

// Reply trait
pub trait Reply {
    fn is_error(&self) -> bool;
    fn to_str(&self) -> Result<String, Box<dyn std::error::Error>>;
}

impl Reply for DiscordMessage {
    fn to_str(&self) -> Result<String, Box<dyn std::error::Error>> {
        let msg_json = serde_json::to_string(self)?;
        Ok(msg_json)
    }
    fn is_error(&self) -> bool {
        false
    }
}

impl Reply for DiscordError {
    fn is_error(&self) -> bool {
        true
    }
    fn to_str(&self) -> Result<String, Box<dyn std::error::Error>> {
        let msg_json = serde_json::to_string(self)?;
        Ok(msg_json)
    }
}

fn read_discord_reply(json: &str) -> Result<Box<dyn Reply>, Box<dyn std::error::Error>> {
    let msg: Box<dyn Reply>;
    let error: DiscordError;
    match serde_json::from_str::<DiscordMessage>(json) {
        Ok(out) => msg = Box::new(out),
        Err(e) => {
            if e.is_data() {
                error = serde_json::from_str::<DiscordError>(json)?;
                return Ok(Box::new(error));
            } else {
                return Err(e)?;
            }
        }
    };
    Ok(msg)
}
