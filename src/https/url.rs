use regex::Regex;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
// parsing urls.

#[derive(Debug)]
pub enum UrlError {
    InvalidUrl(&'static str),
    RegexError,
}
impl Display for UrlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            UrlError::InvalidUrl(t) => write!(f, "{}", t),
            UrlError::RegexError => write!(f, "the regex failed to compile"),
        }
    }
}
impl Error for UrlError {}

pub struct Url<'a> {
    route: &'a str,
    domain: &'a str,
    scheme: &'a str,
    port: u16,
    query: &'a str,
}

impl Debug for Url<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Url")
            .field("route", &self.route)
            .field("domain", &self.domain)
            .field("scheme", &self.scheme)
            .field("port", &self.port)
            .field("query", &self.query)
            .finish()
    }
}

impl<'a> Url<'a> {
    pub fn new(u: &'a str) -> Result<Url<'a>, UrlError> {
        let reg = match Regex::new(
            r"(?<scheme>.*?)://(?<domain>.*?[^\\]?)(?<route>/.*?[^\?]*)(?<query>.*)",
        ) {
            Ok(r) => r,
            Err(_) => return Err(UrlError::RegexError),
        };

        let matches = match reg.captures(u) {
            None => return Err(UrlError::InvalidUrl("couldn't identify the url")),
            Some(o) => o,
        };

        let scheme = match &matches.name("scheme") {
            None => return Err(UrlError::InvalidUrl("no scheme in url")),
            Some(o) => o.as_str(),
        };
        let route = match &matches.name("route") {
            None => "/",
            Some(o) => o.as_str(),
        };
        let domain = match &matches.name("domain") {
            None => return Err(UrlError::InvalidUrl("no domain in url")),
            Some(o) => o.as_str(),
        };
        let query = &matches.name("query").map_or("", |o| o.as_str());
        let port = match scheme {
            "http" => 80,
            "https" => 443,
            "ftp" => 21,
            _ => 1919, // IDK
        };

        Ok(Url {
            route,
            domain,
            scheme,
            port,
            query,
        })
    }
    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.domain, self.port)
    }
    pub fn route(&self) -> &'a str {
        self.route
    }
    pub fn domain(&self) -> &'a str {
        self.domain
    }
    pub fn query(&self) -> &'a str {
        self.query
    }
    pub fn scheme(&self) -> &'a str {
        self.scheme
    }
    pub fn port(&self) -> u16 {
        self.port
    }
}
