use crate::https::https_client::Client;
use crate::https::request::{HTTPMethods, RequestBuilder};
use crate::https::response::Response;
use log::debug;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json;
use std::error::Error;
// This region is used to handle Xbox Live requests and responses.
// from this we will be able to get a XL token and a Userhash.
//

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
struct XboxLiveRequest<'a> {
    Properties: XLR_Properties<'a>,
    RelyingParty: &'static str,
    TokenType: &'static str,
}
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
struct XLR_Properties<'a> {
    AuthMethod: &'static str,
    SiteName: &'static str,
    RpsTicket: &'a str,
}

// Implements a function to serialize a request into Vec<u8>
impl<'a> XboxLiveRequest<'a> {
    pub fn new(token: &str) -> Vec<u8> {
        info!("Created a XboxLive request.");
        let props = XLR_Properties {
            AuthMethod: "RPS",
            SiteName: "user.auth.xboxlive.com",
            RpsTicket: &format!("d={}", token),
        };

        let req = XboxLiveRequest {
            Properties: props,
            RelyingParty: "http://auth.xboxlive.com",
            TokenType: "JWT",
        };

        serde_json::to_vec(&req).unwrap()
    }
}

// Xbox Live Response
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[derive(Deserialize)]
struct XboxLiveResponse {
    IssueInstant: String,
    NotAfter: String,
    Token: String,
    DisplayClaims: Xui,
}

// xui field
#[derive(Deserialize)]
struct Xui {
    xui: Vec<UserHash>,
}
// Array inside xui

#[derive(Deserialize)]
struct UserHash {
    uhs: String,
}

// This region will handle obtaining a XSTS token

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[derive(Serialize)]
struct XSTSReqProperties {
    SandboxId: &'static str,
    UserTokens: Vec<String>,
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[derive(Serialize)]
struct XSTSRequest {
    Properties: XSTSReqProperties,
    RelyingParty: &'static str,
    TokenType: &'static str,
}

impl XSTSRequest {
    pub fn new(xbl_token: &str) -> Vec<u8> {
        info!("Created a XSTS Request.");
        let req = XSTSRequest {
            Properties: XSTSReqProperties {
                SandboxId: "RETAIL",
                UserTokens: vec![xbl_token.to_string()],
            },
            RelyingParty: "rp://api.minecraftservices.com/",
            TokenType: "JWT",
        };

        serde_json::to_vec(&req).unwrap()
    }
}

// This region deals with authentication with Minecraft.
#[derive(Serialize)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
struct MCLogin {
    identityToken: String,
}

#[derive(Deserialize)]
struct MCLoginResponse {
    username: String,
    roles: Vec<String>,
    access_token: String, // JWT
    token_type: String,
    expires_in: usize,
}

impl MCLogin {
    pub fn new(uh: &str, xsts_token: &str) -> Vec<u8> {
        info!("Created a Minecraft Login request.");
        serde_json::to_vec(&MCLogin {
            identityToken: format!("XBL3.0 x={0};{1}", uh, xsts_token),
        })
        .unwrap()
    }
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct Skin {
    id: String,
    state: String,
    url: String,
    variant: String,
    alias: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct Cape {
    id: String,
    state: String,
    url: String,
    alias: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
struct MCProfile {
    id: String,
    name: String,
    Skins: Vec<Skin>,
    Capes: Vec<Cape>,
}

pub fn login_to_minecraft(access_token: &str) -> Result<(String, String, String), Box<dyn Error>> {
    info!("Started login process!");
    let mut client = Client::new("user.auth.xboxlive.com")?;
    let mut buf: [u8; 4096] = [0; 4096];
    let mut response: Response;
    let mut req = RequestBuilder::new()
        .set_method(HTTPMethods::POST)
        .set_route("/user/authenticate")
        .set_host("user.auth.xboxlive.com")
        .add_header("User-Agent: bigeon/0.0.1")
        .add_header("Accept: application/json")
        .add_header("Content-Type: application/json")
        .set_content(&XboxLiveRequest::new(access_token))
        .build();
    debug!("Request: {:?}", std::str::from_utf8(&req)?);
    client.client_write(&req)?;

    let mut len = client.client_read(&mut buf)?;
    info!("Sent XboxLive request to the API.");
    response = Response::from_bytes(&buf[0..len - 1])?;
    println!("{:?}", response);

    let xl_response = serde_json::from_slice::<XboxLiveResponse>(&response.content)?;

    client = Client::new("xsts.auth.xboxlive.com")?;
    req = RequestBuilder::new()
        .set_method(HTTPMethods::POST)
        .set_route("/xsts/authorize")
        .add_header("Accept: application/json")
        .add_header("Content-Type: application/json")
        .set_host("xsts.auth.xboxlive.com")
        .set_content(&XSTSRequest::new(&xl_response.Token))
        .build();

    client.client_write(&req)?;
    len = client.client_read(&mut buf)?;

    response = Response::from_bytes(&buf[0..len - 1])?;
    let xsts_response = serde_json::from_slice::<XboxLiveResponse>(&response.content)?;
    let (xsts_token, userhash) = (xsts_response.Token, &xsts_response.DisplayClaims.xui[0].uhs);

    client = Client::new("api.minecraftservices.com")?;
    req = RequestBuilder::new()
        .set_method(HTTPMethods::POST)
        .set_route("/authentication/login_with_xbox")
        .set_host("api.minecraftservices.com")
        .add_header("Accept: application/json")
        .add_header("Content-Type: application/json")
        .set_content(&MCLogin::new(userhash, &xsts_token))
        .build();

    client.client_write(&req)?;
    len = client.client_read(&mut buf)?;

    response = Response::from_bytes(&buf[0..len - 1])?;
    let mc_response = serde_json::from_slice::<MCLoginResponse>(&response.content)?;

    let jwt = mc_response.access_token;

    req = RequestBuilder::new()
        .set_method(HTTPMethods::GET)
        .add_header(&format!("Authorization: Bearer {}", jwt))
        .add_header("Content-Type: application/json")
        .add_header("Accept: application/json")
        .set_route("/minecraft/profile")
        .set_host("api.minecraftservices.com")
        .build();

    client.client_write(&req)?;
    len = client.client_read(&mut buf)?;

    response = Response::from_bytes(&buf[0..len - 1])?;
    let mc_profile = serde_json::from_slice::<MCProfile>(&response.content)?;
    Ok((jwt, mc_profile.id, mc_profile.name))
}
