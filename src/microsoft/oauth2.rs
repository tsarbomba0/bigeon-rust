use crate::https::client::HttpsClient;
use crate::https::response::Response;
use log::info;
use regex::Regex;
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::net::TcpListener;
use std::str;

const ACCESS_TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";

#[derive(Deserialize)]
struct Oauth2Settings {
    client_secret: String,
    client_id: String,
}

#[derive(Deserialize)]
pub struct MsTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: usize,
    pub scope: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
}

pub fn get_oauth2_code() -> Result<MsTokenResponse, Box<dyn Error>> {
    // read client settings from file
    let mut f = File::open("oauth2.json")?;
    let mut data = vec![];
    f.read_to_end(&mut data)?;
    let settings = serde_json::from_slice::<Oauth2Settings>(&data)?;
    info!("Read Oauth2 settings from oauth2.json");

    // oauth2 route used for logging in
    let oauth2_route: String = format!("consumers/oauth2/v2.0/authorize?client_id={}&response_type=code&redirect_uri=http://localhost:6636&scope=XboxLive.signin&response_mode=query&state=wersal", settings.client_id);
    // regex for code and status
    let re = Regex::new(r"code=(?<code>.*?[^&]*)&state=(?<state>\w{6})")?;

    // login
    println!(
        "Log in here: https://login.microsoftonline.com/{}",
        oauth2_route
    );

    info!("Starting oauth2 chain!");

    // tcp server to redirect to
    let server = TcpListener::bind("127.0.0.1:6636")?;
    let mut buf = Vec::new();
    let mut conn = server.accept()?;
    conn.0.read_to_end(&mut buf)?;
    let mut redirect = str::from_utf8(&buf)?.split_whitespace();

    // obtains the route after GET
    let response_route = match redirect.next() {
        Some(method) => {
            if method != "GET" {
                panic!("Sent response was NOT a GET request.");
            }
            match redirect.next() {
                None => panic!("No request URI was sent!"),
                Some(r) => r,
            }
        }
        None => panic!("No valid response was sent."),
    };

    // checks if there is anything present
    let captures = match re.captures(response_route) {
        None => panic!("Couldn't find either code nor state in the response!"),
        Some(o) => o,
    };

    // captures the ode
    let code = match captures.name("code") {
        None => panic!("No oauth2 code obtained."),
        Some(c) => c.as_str(),
    };
    println!("{:?}", captures);
    match captures.name("state") {
        None => panic!("No state was received!"),
        Some(st) => {
            if st.as_str() != "wersal" {
                println!("{}", &st.as_str());
            };
        }
    }

    // post form with our obtained code
    let access_token_post_form: String  = format!("client_id={}&scope=XboxLive.signin&redirect_uri=http://localhost:6636&grant_type=authorization_code&code={}&client_secret={}", settings.client_id, code, settings.client_secret);
    info!("Obtained code! Getting the access token");

    let mut client = HttpsClient::new("Bigeon/0.0.2", None);
    let mut headers: HashMap<&str, &str> = HashMap::new();
    headers.insert("Content-Type", "application/x-www-form-urlencoded");

    let bytes = client
        .post(
            ACCESS_TOKEN_URL,
            access_token_post_form.into_bytes(),
            Some(headers),
        )
        .unwrap();
    let response = Response::from_slice(&bytes).unwrap();
    let token_struct = serde_json::from_slice::<MsTokenResponse>(&response.content)?;

    info!("Obtained Access token!");

    Ok(token_struct)
}
