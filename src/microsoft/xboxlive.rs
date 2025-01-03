use crate::https::client::HttpsClient;
use crate::https::persistent_client::PersistentClient;
use crate::https::response::Response;
use log::debug;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::error::Error;
// This region is used to handle Xbox Live requests and responses.
// from this we will be able to get a XL token and a Userhash.
//

const USER_AGENT: &str = "bigeon/0.0.2";
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
    // headers to say we want json data and send json data
    let json_headers = vec![
        ("Accept", "application/json"),
        ("Content-Type", "application/json"),
    ]
    .into_iter()
    .collect::<HashMap<&str, &str>>();
    info!("Started login process!");

    // client and response
    let mut client = HttpsClient::new(USER_AGENT, Some(&json_headers));
    let mut response: Response;

    // xboxlive
    let reply = client.post(
        "https://user.auth.xboxlive.com/user/authenticate",
        XboxLiveRequest::new(access_token),
        None,
    )?.;
    response = Response::from_slice(&reply)?;
    info!("Sent XboxLive request to the API.");
    let xl_response = serde_json::from_slice::<XboxLiveResponse>(&response.content)?;
    info!("Received reply from XboxLive!");
    drop(reply);

    // xsts
    let reply = client.post(
        "https://xsts.auth.xboxlive.com/xsts/authorize",
        XSTSRequest::new(&xl_response.Token),
        None,
    )?;
    info!("Sending request to XSTS!");
    response = Response::from_slice(&reply)?;

    let xsts_response = serde_json::from_slice::<XboxLiveResponse>(&response.content)?;
    let (xsts_token, userhash) = (xsts_response.Token, &xsts_response.DisplayClaims.xui[0].uhs);
    info!("Got response from XSTS!");

    drop(client);

    let mut client =
        PersistentClient::new(USER_AGENT, Some(json_headers), "api.minecraftservices.com")?;

    // login with xbox -> minecraft
    info!("Obtaining login info for minecraft!");
    let reply = client.post(
        "https://api.minecraftservices.com/authentication/login_with_xbox",
        &MCLogin::new(&userhash, &xsts_token),
        None,
    )?;
    response = Response::from_slice(&reply)?;

    let mc_response = serde_json::from_slice::<MCLoginResponse>(&response.content)?;
    let jwt = mc_response.access_token;

    // get minecraft profile
    info!("Fetching minecraft profile!");
    let bearer = format!("Bearer: {}", jwt);
    let header = vec![("Authorization", bearer.as_str())]
        .into_iter()
        .collect::<HashMap<&str, &str>>();

    let reply = client.get(
        "https://api.minecraftservices.com/minecraft/profile",
        Some(header),
    )?;
    response = Response::from_slice(&reply)?;
    let mc_profile = serde_json::from_slice::<MCProfile>(&response.content)?;
    info!("Fetched minecraft profile: {}", mc_profile.name);
    Ok((jwt, mc_profile.id, mc_profile.name))
}
