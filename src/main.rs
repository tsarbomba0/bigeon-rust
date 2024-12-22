use crate::discord::{client, message};
use crate::https::https_client::Client;
use crate::https::request::RequestBuilder;
use discord::client::DiscordClient;
use discord::message::DiscordMessage;
use discord::message::Reply;
use tokio::spawn;
mod discord;
mod https;

fn main() {
    let mut client = DiscordClient::new("");

    let msg = message::MessageBuilder::new()
        .content("Welcome from Rust!")
        .build();

    let response = client.send_message(msg, "1296137217604849704");
    println!("{:?}", response.unwrap())
}
