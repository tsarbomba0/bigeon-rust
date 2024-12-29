use log::debug;
use std::collections::HashMap;
use std::fmt;
use std::str;

pub struct Response {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub content: Vec<u8>,
}

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Response {{\nstatus_code: {0},\nheaders:{1:#?},\ncontent:{2:?}\n}}",
            self.status_code, self.headers, self.content
        )
    }
}

impl Response {
    pub fn from_bytes(data: &[u8]) -> Result<Response, Box<dyn std::error::Error>> {
        let data_vec = data.to_vec();
        let mut iter = data_vec.iter();
        // booleans for parsing
        let mut cr = false;
        let mut lf = false;
        let mut second_cr = false;

        // Lines
        let mut lines: Vec<String> = Vec::new();
        let mut buf: Vec<u8> = Vec::new();

        // Headers
        let mut headers: HashMap<String, String> = HashMap::new();

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
                        buf.push(*byte);
                        (cr, lf) = (false, false)
                    } else {
                        buf.push(*byte);
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
                    headers.insert(field.to_string(), value.trim().to_string())
                }
            };
        }

        let content_len = match headers.get("Content-Length") {
            Some(length) => {
                let content_length = length.parse::<usize>()?;
                for byte in iter.by_ref() {
                    content.push(*byte);
                    if content.len() == content_length {
                        break;
                    }
                }
                Some(content.len())
            }
            None => None,
        };

        let mut transfer_encoding = "none";
        if content_len.is_none() {
            match headers.get("Transfer-Encoding") {
                None => {
                    debug!("Can't determine length of content, reading everything.");
                    for byte in iter.by_ref() {
                        content.push(*byte)
                    }
                    return Ok(Response {
                        status_code,
                        headers,
                        content,
                    });
                }
                Some(v) => transfer_encoding = v.as_str(),
            }
        }

        match transfer_encoding {
            "chunked" => {
                let mut data_read = false;
                let mut length_read = true;
                let mut read_length = 0;

                let mut len = content.len();
                let mut buf = Vec::new();

                while let Some(byte) = iter.next() {
                    if data_read {
                        if len + read_length == content.len() {
                            data_read = false;
                            iter.nth(0);
                            len = content.len();
                            dbg!(len);
                        } else {
                            content.push(*byte)
                        }
                    }
                    if length_read {
                        match byte {
                            b'\r' => {
                                iter.nth(0);
                                length_read = false;
                                data_read = true;
                                read_length = usize::from_str_radix(str::from_utf8(&buf)?, 16)?;
                                dbg!(read_length);
                                if read_length == 0 {
                                    break;
                                }
                            }
                            _ => {
                                buf.push(*byte);
                            }
                        }
                    }
                }
            }
            "none" => (),
            _ => unimplemented!(),
        }

        Ok(Response {
            status_code,
            headers,
            content,
        })
    }
}
