#[allow(dead_code)]
pub enum HTTPMethods {
    POST,
    HEAD,
    GET,
    PUT,
    DELETE,
}

pub struct RequestBuilder<'a> {
    method: HTTPMethods,
    route: Option<&'a str>,
    host: Option<&'a str>,
    pub headers: Vec<&'a str>,
    pub content: Option<&'a [u8]>,
    content_len: usize,
}

impl<'a> RequestBuilder<'a> {
    pub fn new() -> Self {
        Self {
            method: HTTPMethods::GET,
            route: None,
            host: None,
            headers: Vec::new(),
            content: None,
            content_len: 0,
        }
    }

    pub fn set_method(&mut self, method: HTTPMethods) -> &mut Self {
        self.method = method;
        self
    }
    pub fn set_route(&mut self, route: &'a str) -> &mut Self {
        self.route = Some(route);
        self
    }
    pub fn set_host(&mut self, host: &'a str) -> &mut Self {
        self.host = Some(host);
        self
    }
    pub fn add_header(&mut self, header: &'a str) -> &mut Self {
        self.headers.push(header);
        self
    }
    pub fn add_many_headers(&mut self, headers: &'a Vec<String>) -> &mut Self {
        for header in headers {
            self.headers.push(header);
        }
        self
    }
    pub fn set_content(&mut self, content: &'a [u8]) -> &mut Self {
        self.content = Some(content);
        self.content_len = content.len();
        self
    }
    fn crlf(&mut self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&[13, 10]);
    }
    pub fn build(&mut self) -> Vec<u8> {
        let mut buf = Vec::new();
        // Method and white space
        match self.method {
            HTTPMethods::POST => buf.extend_from_slice(&[80, 79, 83, 84]),
            HTTPMethods::PUT => buf.extend_from_slice(&[72, 69, 65, 68]),
            HTTPMethods::GET => buf.extend_from_slice(&[71, 69, 84]),
            HTTPMethods::DELETE => buf.extend_from_slice(&[80, 85, 84]),
            HTTPMethods::HEAD => buf.extend_from_slice(&[68, 69, 76, 69, 84, 69]),
        };
        buf.push(32);

        // Route and white space
        buf.extend_from_slice(self.route.unwrap().as_bytes());
        buf.push(32);

        // HTTP/1.1 and white space
        buf.extend_from_slice(&[72, 84, 84, 80, 47, 49, 46, 49]);
        self.crlf(&mut buf);

        // Host
        buf.extend_from_slice(format!("Host: {}", self.host.unwrap()).as_bytes());
        self.crlf(&mut buf);

        // Headers
        for header in &mut *self.headers {
            buf.extend_from_slice(header.as_bytes());
            buf.extend_from_slice(&[13, 10]);
        }
        // content length header
        buf.extend_from_slice(format!("Content-Length: {}", self.content_len).as_bytes());
        self.crlf(&mut buf);

        // Content
        self.crlf(&mut buf);

        if let Some(v) = self.content {
            buf.extend_from_slice(v);
        };
        buf
    }
}
