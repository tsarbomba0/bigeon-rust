use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::str;

pub struct Response<'a> {
    pub status_code: String,
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
        println!("{}", str::from_utf8(data)?);
        let iter = data.to_vec().into_iter();

        // booleans for parsing
        let mut cr = false;
        let mut lf = false;
        let mut second_cr = false;
        let mut second_lf = false;

        // Lines
        let mut lines_vec: Vec<Vec<u8>> = vec![Vec::new()];
        let mut count = 0;

        // Content
        let mut content: Vec<u8> = Vec::new();
        // split bytes into lines
        for byte in iter {
            match byte {
                10 => {
                    if lf {
                        second_lf = true;
                    } else {
                        lf = true
                    }
                }

                13 => {
                    if cr {
                        second_cr = true;
                    } else {
                        cr = true;
                    }
                }

                _ => {
                    if second_lf && second_cr {
                        content.push(byte);
                    } else if cr && lf {
                        lines_vec.push(Vec::new());
                        count += 1;
                        lines_vec[count].push(byte);
                        (cr, lf) = (false, false)
                    } else {
                        lines_vec[count].push(byte)
                    }
                }
            }
        }

        // Status code
        let mut lines = lines_vec.into_iter();
        let status_code: String;
        match &lines.next() {
            Some(line) => status_code = str::from_utf8(&line[9..12])?.to_owned(),
            None => return Err("Empty response")?,
        };

        let mut headers: HashMap<Cow<'a, str>, String> = HashMap::new();
        // Headers
        for line in lines {
            let mut header_iter = str::from_utf8(&line)?.split(":");
            match header_iter.next() {
                None => break,
                Some(field) => {
                    let Some(value) = header_iter.next() else {
                        return Err("Invalid header entry")?;
                    };
                    headers.insert(Cow::Owned(field.to_string()), value.to_owned())
                }
            };
        }
        Ok(Response {
            status_code,
            headers,
            content,
        })
    }
}
