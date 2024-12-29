use discord::client::DiscordClient;
use discord::message::MessageBuilder;
mod discord;
mod https;

use minecraft::microsoft::oauth2::get_oauth2_code;
use minecraft::microsoft::oauth2::MsTokenResponse;
use minecraft::microsoft::xboxlive::login_to_minecraft;
mod minecraft;

use log::{Level, LevelFilter, Metadata, Record};

struct BigeonLogger;
static BIGEON_LOGGER: BigeonLogger = BigeonLogger;

impl log::Log for BigeonLogger {
    fn flush(&self) {}

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{}: {} ", record.level(), record.args());
        }
    }

    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }
}

fn main() {
    // Logger
    log::set_logger(&BIGEON_LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Debug))
        .unwrap();

    let mut cl = DiscordClient::new("");
    let message = MessageBuilder::new().content("Ahaha!").build();

    cl.send_message(message, "1296137217604849704").unwrap();

    let token_struct = get_oauth2_code().unwrap();
    let (jwl, uuid, name) = login_to_minecraft(&token_struct.access_token).unwrap();
    println!("JWL: {}, UUID: {}, NAME: {}", jwl, uuid, name);
}
