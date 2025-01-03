use super::client::Methods;
use super::request::RequestBuilder;
use super::url::{Url, UrlError};
use crate::https::canbeclient::CanBeClient;
use crate::tls::tls_stream::TlsStream;
use std::collections::HashMap;
use std::io::{Error, Read, Write};

type TLSResult<T> = Result<T, Error>;
type HeaderMap<'a> = HashMap<&'a str, String>;
pub struct PersistentClient<'p> {
    io: TlsStream,
    head: HeaderMap<'p>,
}

impl<'p> PersistentClient<'p> {
    pub fn new(a: &'p str, url: &'p str) -> TLSResult<Self> {
        let p_url = Url::new(url).unwrap();

        Ok(Self {
            io: TlsStream::new(None, p_url.domain(), &p_url.socket_addr())?,
            head: HashMap::from_iter(vec![("User-Agent", a.to_string())]),
        })
    }

    pub fn default_headers(&mut self, headers: HashMap<&'p str, String>) {
        for (k, v) in headers.into_iter() {
            self.head.insert(k, v);
        }
    }

    pub fn io_write(&mut self, buf: &[u8]) -> TLSResult<usize> {
        self.io.write(buf)
    }

    pub fn io_read(&mut self) -> TLSResult<Vec<u8>> {
        let mut buf = vec![];
        self.io.read_to_end(&mut buf)?;
        Ok(buf)
    }

    pub fn get(&'p mut self, url: &'p str) -> Result<RequestBuilder<'p>, UrlError> {
        self.request(Methods::GET, url)
    }

    pub fn post(&'p mut self, url: &'p str) -> Result<RequestBuilder<'p>, UrlError> {
        self.request(Methods::POST, url)
    }
}

impl CanBeClient<'_> for PersistentClient<'_> {
    fn request<'a>(&'a mut self, m: Methods, url: &'a str) -> Result<RequestBuilder<'a>, UrlError> {
        let p_url = Url::new(url)?;
        let req = RequestBuilder::new(p_url);

        Ok(req.http_method(m).headers(&self.head))
    }
}
