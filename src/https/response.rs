use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::str;
pub struct Response<'a> {
    pub status_code: u16,
    pub headers: HashMap<Cow<'a, str>, String>,
    pub content: Vec<u8>,
}

impl fmt::Debug for Response<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Response {{\nstatus_code: {0},\nheaders:{1:#?},\ncontent:{2:?}\n}}",
            self.status_code, self.headers, self.content
        )
    }
}

impl<'a> Response<'a> {
    pub fn from_bytes(data: &'a [u8]) -> Result<Response<'a>, Box<dyn std::error::Error>> {
        let data_vec = data.to_vec();
        let mut iter = data_vec.into_iter();

        // booleans for parsing
        let mut cr = false;
        let mut lf = false;
        let mut second_cr = false;

        // Lines
        let mut lines: Vec<String> = Vec::new();
        let mut buf: Vec<u8> = Vec::new();

        // Headers
        let mut headers: HashMap<Cow<'a, str>, String> = HashMap::new();

        // Content
        let mut content: Vec<u8> = Vec::new();

        // split bytes into lines
        while let Some(byte) = iter.next() {
            match byte {
                b'\n' => {
                    // break if we are on a second CR
                    // as in \r\n\r **\n**
                    if second_cr {
                        break;
                    };

                    // match for earlier characters
                    match lf {
                        true => (),
                        false => lf = true,
                    }
                }

                b'\r' => {
                    if cr {
                        second_cr = true;
                    } else {
                        cr = true;
                    }
                }

                _ => {
                    if cr && lf {
                        {
                            lines.push(str::from_utf8(&buf)?.to_string());
                            buf.clear();
                        };
                        buf.push(byte);
                        (cr, lf) = (false, false)
                    } else {
                        buf.push(byte);
                    }
                }
            }
        }

        // Iterator over lines
        let mut lines_iter = lines.iter();

        // Status code
        let status_code = match &lines_iter.next() {
            Some(line) => line[9..12].parse::<u16>()?,
            None => return Err("Empty response")?,
        };

        // Headers
        for line in lines_iter {
            let mut header = line.split(":");
            match header.next() {
                None => break,
                Some(field) => {
                    let Some(value) = header.next() else {
                        return Err("Invalid header entry")?;
                    };
                    headers.insert(Cow::Owned(field.to_string()), value.trim().to_owned())
                }
            };
        }

        // Content-Length extraction
        // if it's not provided, it's going to be set to -1
        match headers.get("Content-Length") {
            Some(length) => {
                let content_length = length.parse::<usize>()?;
                for byte in iter {
                    content.push(byte);
                    if content.len() == content_length {
                        break;
                    }
                }
            }
            None => {
                for byte in iter {
                    content.push(byte)
                }
            }
        };

        Ok(Response {
            status_code,
            headers,
            content,
        })
    }
}
