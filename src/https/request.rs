use crate::https::client::Methods;
use crate::https::persistent_client::PersistentClient;
use crate::https::url::Url;
use std::collections::HashMap;
use std::error::Error;

const CRLF: &[u8] = "\r\n".as_bytes();

pub struct RequestBuilder<'a> {
    method: Methods,
    route: &'a str,
    headers: HashMap<&'a str, &'a str>,
    content: Option<&'a [u8]>,
    content_len: usize,
    host: &'a str,
}

impl<'a> RequestBuilder<'a> {
    pub fn new(url: Url<'a>) -> Self {
        Self {
            method: Methods::GET,
            route: url.route(),
            headers: HashMap::new(),
            content: None,
            content_len: 0,
            host: url.domain(),
        }
    }
    pub fn http_method(mut self, m: Methods) -> Self {
        self.method = m;
        self
    }
    pub fn header(mut self, h: (&'a str, &'a str)) -> Self {
        self.headers.insert(h.0, h.1);
        self
    }
    pub fn headers(mut self, h: &'a HashMap<&str, String>) -> Self {
        for (key, val) in h {
            self.headers.insert(key, &val);
        }
        self
    }
    pub fn content(mut self, c: &'a [u8]) -> Self {
        self.content_len = c.len();
        self.content = Some(c);
        self
    }

    pub fn execute(self, exec: &'a mut PersistentClient<'a>) -> Result<Vec<u8>, std::io::Error> {
        let mut buf = vec![];

        let b_method = match self.method {
            Methods::GET => "GET".as_bytes(),
            Methods::POST => "POST".as_bytes(),
            Methods::PATCH => "PATCH".as_bytes(),
            Methods::OPTIONS => "OPTIONS".as_bytes(),
            Methods::CONNECT => "CONNECT".as_bytes(),
            Methods::HEAD => "HEAD".as_bytes(),
            Methods::PUT => "PUT".as_bytes(),
            Methods::DELETE => "DELETE".as_bytes(),
        };

        buf.extend_from_slice(b_method);
        buf.extend_from_slice(&[32]);

        // route
        buf.extend_from_slice(self.route.as_bytes());
        buf.push(32);

        buf.extend_from_slice("HTTP/1.1".as_bytes());
        buf.extend_from_slice(CRLF);

        for (k, v) in &self.headers {
            buf.extend_from_slice(k.as_bytes());
            buf.extend_from_slice(": ".as_bytes());
            buf.extend_from_slice(v.as_bytes());
            buf.extend_from_slice(CRLF);
        }
        buf.extend_from_slice(CRLF);

        // Content-Length and Content
        if self.content_len > 0 {
            buf.extend_from_slice("Content-Length: ".as_bytes());
            buf.extend_from_slice(format!("{}\r\n", self.content_len).as_bytes());
            if let Some(c) = self.content {
                buf.extend_from_slice(&c);
            };
        };

        exec.io_write(&buf);
        exec.io_read()
    }
}
