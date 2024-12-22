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
    pub fn from_bytes(data: &[u8]) -> Result<Response, Box<dyn std::error::Error>> {
        let mut iter = data.iter();

        // booleans for parsing
        let mut cr = false;
        let mut lf = false;
        let mut second_cr = false;
        let mut second_lf = false;

        // Buffer for content of the response
        let content_buf: Vec<u8>;

        // Lines
        let lines: Vec<&str>;
        // Buffer for bytes
        let buf: Vec<u8>;

        for byte in iter {
            match byte {
                // Carriage return
                13 => {
                    if cr {
                        second_cr = true;
                    } else {
                        cr = true
                    }
                }
                // Line feed
                10 => {
                    if lf {
                        second_lf = true;
                    } else {
                        lf = true;
                    }
                }
                _ => {
                    if cr && lf {
                        lines.push(std::str::from_utf8(&buf)?);
                        buf.clear();
                        (cr, lf) = (false, false)
                    } else {
                        buf.push(byte.to_owned())
                    }
                }
            }
        }
        Ok(Response {})
    }
}
