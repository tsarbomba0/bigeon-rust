use log::{debug, info, warn};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::io::Error;
use std::io::ErrorKind;
use std::rc::Rc;
use std::str;

fn read_chunk_length(buf: &[u8]) -> Result<usize, Error> {
    let str = match str::from_utf8(buf) {
        Ok(o) => o,
        Err(_) => return Err(Error::from(ErrorKind::InvalidInput)),
    };
    match usize::from_str_radix(str, 16) {
        Ok(num) => Ok(num),
        Err(_) => Err(Error::from(ErrorKind::InvalidInput)),
    }
}

pub struct Response {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub content: Vec<u8>,
}

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Response {{    \nstatus_code: {0},    \nheaders:{1:#?},    \ncontent:{2:?}\n}}",
            self.status_code, self.headers, self.content
        )
    }
}

impl Response {
    pub fn from_bytes(data: &[u8]) -> Result<Response, Error> {
        let mut iter = data.iter();
        // booleans for parsing
        let mut cr = false;
        let mut lf = false;
        let mut second_cr = false;

        // lines
        let mut lines: Vec<String> = Vec::new();
        let buf: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));

        // headers
        let mut headers: HashMap<String, String> = HashMap::new();

        // content
        let mut content: Vec<u8> = Vec::new();

        // split bytes into lines
        while let Some(byte) = iter.next() {
            match byte {
                // line feed/newline
                b'\n' => {
                    // break if we are on a second CR
                    // as in \r\n\r **\n**
                    if second_cr {
                        // this ensures we put the last line to our lines vector
                        let rc_buf = buf.borrow();
                        let line = match str::from_utf8(&rc_buf) {
                            Ok(str) => Some(str),
                            Err(_) => {
                                warn!("Incorrect UTF-8 in header!");
                                None
                            }
                        };

                        if let Some(l) = line {
                            lines.push(l.to_string());
                        };
                        break;
                    };

                    // match for earlier characters
                    match lf {
                        true => (),
                        false => lf = true,
                    }
                }

                // carriage return
                b'\r' => match cr {
                    true => second_cr = true,
                    false => cr = true,
                },

                // any other character
                _ => {
                    if cr && lf {
                        let mut rc_buf = buf.borrow_mut();
                        let line = match str::from_utf8(&rc_buf) {
                            Ok(str) => Some(str),
                            Err(_) => {
                                warn!("Incorrect UTF-8 in header!");
                                None
                            }
                        };

                        if let Some(l) = line {
                            lines.push(l.to_string());
                        };
                        rc_buf.clear();
                        rc_buf.push(*byte);
                        (cr, lf) = (false, false)
                    } else {
                        buf.borrow_mut().push(*byte);
                    }
                }
            }
        }

        // iterator over lines
        let mut lines_iter = lines.iter();

        // Status code
        let status_code = match &lines_iter.next() {
            Some(line) => match line[9..12].parse::<u16>() {
                Ok(str) => str,
                Err(_) => return Err(Error::from(ErrorKind::InvalidData)),
            },
            None => return Err(Error::from(ErrorKind::UnexpectedEof)),
        };

        // Headers
        for line in lines_iter {
            let mut header = line.split(":");
            match header.next() {
                None => break,
                Some(field) => {
                    let Some(value) = header.next() else {
                        warn!("Incorrect header, skipping it.");
                        continue;
                    };
                    headers.insert(field.to_string(), value.trim().to_string())
                }
            };
        }

        // content length
        let content_len = match headers.get("Content-Length") {
            Some(length) => {
                let content_length = match length.parse::<usize>() {
                    Ok(o) => o,
                    Err(_) => return Err(Error::from(ErrorKind::InvalidData)),
                };

                for byte in iter.as_ref() {
                    content.push(*byte);
                    if content.len() == content_length {
                        break;
                    }
                }
                Some(content.len())
            }
            None => {
                info!("No Content-Length header present!");
                None
            }
        };

        // transfer encoding
        let mut transfer_encoding = "none";
        if content_len.is_none() {
            match headers.get("Transfer-Encoding") {
                None => {
                    warn!("Can't determine length of content, reading everything.");
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
                                read_length = read_chunk_length(&buf)?;
                                debug!("Chunk length: {}", read_length);
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
