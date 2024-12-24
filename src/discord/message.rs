use core::fmt;
use core::fmt::Debug;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_with::{serde_as, skip_serializing_none};
use std::error::Error;

pub struct DiscordEmbed {}

#[derive(Debug, Deserialize, Serialize)]
pub struct DiscordError {
    code: u32,
    errors: Vec<String>, // todo
    message: String,
}

#[serde_as]
#[skip_serializing_none]
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

impl DiscordMessage {
    pub fn to_vec(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(serde_json::to_vec(&self)?)
    }
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

    pub fn build(&self) -> DiscordMessage {
        DiscordMessage {
            content: self.content.to_owned(),
            nonce: self.nonce.to_owned(),
            tts: self.tts.to_owned(),
            embeds: self.embeds.to_owned(),
            allowed_mentions: self.allowed_mentions.to_owned(),
            message_reference: self.message_reference.to_owned(),
            components: self.components.to_owned(),
            sticker_ids: self.sticker_ids.to_owned(),
            attachments: self.attachments.to_owned(),
            flags: self.flags.to_owned(),
            enforce_nonce: self.enforce_nonce.to_owned(),
            poll: self.poll.to_owned(),
        }
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
    fn to_str(&self) -> Result<String, Box<dyn Error>>;
}

impl Debug for dyn Reply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_error() {
            return write!(f, "DiscordError");
        } else {
            return write!(f, "DiscordMessage");
        }
    }
}

impl Reply for DiscordMessage {
    fn to_str(&self) -> Result<String, Box<dyn Error>> {
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
    fn to_str(&self) -> Result<String, Box<dyn Error>> {
        let msg_json = serde_json::to_string(self)?;
        Ok(msg_json)
    }
}

pub fn read_discord_reply(json: &str) -> Result<Box<dyn Reply>, Box<dyn std::error::Error>> {
    println!("{}", json);
    let msg: Box<dyn Reply>;
    let error: DiscordError;
    match serde_json::from_str::<DiscordMessage>(json) {
        Ok(out) => msg = Box::new(out),
        Err(e) => {
            if e.is_data() {
                let d = serde_json::from_str::<DiscordError>(json).unwrap();
                return Ok(Box::new(d));
            } else {
                return Err(e).unwrap();
            }
        }
    };
    Ok(msg)
}
