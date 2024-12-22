use std::collections::HashMap;
use std::fmt;

pub struct Response {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub content: String,
}

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Response {{\nstatus_code: {0},\nheaders:{1:#?},\ncontent:{2}\n}}",
            self.status_code, self.headers, self.content
        )
    }
}

impl Response {
    pub from_bytes() 
}
