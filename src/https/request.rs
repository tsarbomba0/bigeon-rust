use std::fmt::Write;

pub enum HTTPMethods {
    POST,
    HEAD,
    GET,
    PUT,
    DELETE,
}

pub struct Request {
    method: String,
    route: String,
    host: String,
    pub headers: Vec<String>,
    pub content: String,
}

pub struct RequestBuilder {
    method: Option<HTTPMethods>,
    route: Option<String>,
    host: Option<String>,
    pub headers: Vec<String>,
    pub content: Option<String>,
}

impl RequestBuilder {
    pub fn new() -> Self {
        Self {
            method: None,
            route: None,
            host: None,
            headers: vec![],
            content: None,
        }
    }

    pub fn set_method(&mut self, method: HTTPMethods) -> &mut Self {
        self.method = Some(method);
        self
    }
    pub fn set_route(&mut self, route: &str) -> &mut Self {
        self.route = Some(route.to_owned());
        self
    }
    pub fn set_host(&mut self, host: &str) -> &mut Self {
        self.host = Some(host.to_owned());
        self
    }
    pub fn add_header(&mut self, header: &str) -> &mut Self {
        self.headers.push(header.to_owned());
        self
    }
    pub fn add_many_headers(&mut self, headers: Vec<&str>) -> &mut Self {
        for header in headers {
            self.headers.push(header.to_owned());
        }
        self
    }
    pub fn set_content(&mut self, content: String) -> &mut Self {
        self.content = Some(content);
        self
    }
    pub fn build(&self) -> Request {
        let method = match self.method.as_ref().unwrap() {
            HTTPMethods::POST => "POST",
            HTTPMethods::PUT => "PUT",
            HTTPMethods::GET => "GET",
            HTTPMethods::DELETE => "DELETE",
            HTTPMethods::HEAD => "HEAD",
        };
        let content = match &self.content {
            Some(c) => c,
            None => &"".to_string(),
        };
        Request {
            method: method.to_owned(),
            route: self.route.to_owned().unwrap(),
            host: self.host.to_owned().unwrap(),
            headers: self.headers.to_owned(),
            content: content.to_owned(),
        }
    }
}

impl Request {
    pub fn process(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut output = String::new();
        write!(
            output,
            "{0} {1} HTTP/1.1\r\nHost: {2}\r\n",
            self.method, self.route, self.host
        )?;
        for header in &self.headers {
            output.push_str(header);
            output.push_str("\r\n");
        }

        if !self.content.is_empty() {
            output.push_str(&self.content);
            output.push_str("\r\n\r\n");
        } else {
            output.push_str("\r\n");
        }
        Ok(output)
    }
}
