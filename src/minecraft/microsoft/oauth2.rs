use log::info;
use regex::Regex;
use std::error::Error;
use std::io::Read;
use std::net::TcpListener;
use std::str;

const OAUTH2_ROUTE: &str = "consumers/oauth2/v2.0/authorize?client_id=9da17c9e-e5f1-4d0c-8d1c-6ae1159511c1&response_type=code&redirect_uri=http://localhost:6636&scope=XboxLive.signin&response_mode=query&state=wersal";

pub fn get_oauth2_code() -> Result<String, Box<dyn Error>> {
    let re = Regex::new(r"code=(?<code>.*?[^&]*)&state=(?<state>\w{6})")?;

    // Login
    println!(
        "Log in here: https://login.microsoftonline.com/{}",
        OAUTH2_ROUTE
    );
    let server = TcpListener::bind("127.0.0.1:6636")?;

    info!("Starting oauth2 chain!");
    // Connection
    let mut buf = Vec::new();
    let mut conn = server.accept()?;
    conn.0.read_to_end(&mut buf)?;

    let mut response = str::from_utf8(&buf)?.split_whitespace();

    let response_route = match response.next() {
        Some(method) => {
            if method != "GET" {
                panic!("Sent response was NOT a GET request.");
            }
            match response.next() {
                None => panic!("No request URI was sent!"),
                Some(r) => r,
            }
        }
        None => panic!("No valid response was sent."),
    };

    println!("{}", response_route);
    let captures = match re.captures(response_route) {
        None => panic!("Couldn't find either code nor state in the response!"),
        Some(o) => o,
    };

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
    info!("oauth2 chain finished!");
    Ok(code.to_owned())
}
