use crate::https::response::Response;
use std::collections::HashMap;
use std::str;
// TODO:
// Make this work for byte data too.

pub fn parse_http(resp: &[u8]) -> Result<Response, Box<dyn std::error::Error>> {
    let mut resp_string = match str::from_utf8(resp) {
        Ok(resp) => resp.split("\n"),
        Err(error) => return Err(error)?,
    };
    // Response code
    let response_code = match resp_string.next() {
        Some(value) => {
            let mut split_value = value.split_whitespace();
            split_value.nth(1).unwrap().parse::<u16>()?
        }
        None => return Err("No response code!")?,
    };

    // Values of the response
    let mut headers: HashMap<String, String> = HashMap::new();
    let mut content = String::new();

    // Gathering headers
    for value in &mut resp_string {
        if value == "\r" {
            break;
        } else {
            let mut header = value.split(":");
            let header_name = header.next().unwrap();
            let header_value = match header.next() {
                Some(h) => h.trim(),
                None => return Err("Empty header obtained!")?,
            };
            headers.insert(header_name.to_string(), header_value.to_string());
        }
    }

    for value in resp_string {
        content.push_str(value);
        content.push('\n');
    }

    Ok(Response {
        status_code: response_code,
        headers,
        content,
    })
}
