//use crate::https::response::Response;
use log::debug;
use rustls::ClientConfig;
use rustls::ClientConnection;
use rustls::RootCertStore;
use std::error::Error;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

// Client struct
pub struct Client {
    pub(crate) server_name: String,
    rustls_client: ClientConnection,
    buf_reader: BufReader<TcpStream>,
    buf_writer: BufWriter<TcpStream>,
    tcp_stream: TcpStream,
}

// Client functions.
impl Client {
    pub fn new(server: &str) -> Result<Self, Box<dyn Error>> {
        let cfg = ClientConfig::builder()
            .with_root_certificates(RootCertStore {
                roots: webpki_roots::TLS_SERVER_ROOTS.into(),
            })
            .with_no_client_auth();

        let server_name = String::from(server);
        let dns_name = match server_name.try_into() {
            Ok(name) => {
                println!("Connecting to: {}", server);
                name
            }
            Err(error) => panic!("Incorrect server address. Error: {}", error),
        };
        let conn = match ClientConnection::new(Arc::new(cfg), dns_name) {
            Ok(c) => c,
            Err(error) => {
                panic!("Failed to create a TLS Client Connection, error: {}", error)
            }
        };

        let tcp_stream = TcpStream::connect(server.to_string() + ":443").unwrap();
        Ok(Self {
            rustls_client: conn,
            server_name: server.to_owned(),
            buf_reader: BufReader::new(TcpStream::try_clone(&tcp_stream)?),
            buf_writer: BufWriter::new(TcpStream::try_clone(&tcp_stream)?),
            tcp_stream,
        })
    }

    // Writes to the connection
    pub fn client_write(&mut self, stuff: &[u8]) -> Result<usize, Box<dyn Error>> {
        self.buf_complete_io()?;
        println!("{}", std::str::from_utf8(stuff)?);

        let written = match self.rustls_client.writer().write(stuff) {
            Err(error) => return Err(error)?,
            Ok(n) => n,
        };

        self.rustls_client.complete_io(&mut self.tcp_stream)?;
        dbg!(written);
        Ok(written)
    }

    // Reads from the connection
    pub fn client_read(&mut self, o: &mut [u8]) -> Result<usize, Box<dyn Error>> {
        if self.rustls_client.wants_write() {
            self.buf_complete_io()?;
        }
        if self.rustls_client.is_handshaking() {
            self.buf_complete_io()?;
        }

        while self.rustls_client.wants_read() {
            if self.buf_complete_io()?.0 == 0 {
                break;
            }
        }
        Ok(self.rustls_client.reader().read(o)?)
    }

    // implemntation of rustls' complete_io using buffered io.
    // ugh
    fn buf_complete_io(&mut self) -> Result<(usize, usize), Box<dyn Error>> {
        let mut eof = false;
        let mut read = 0;
        let mut write = 0;

        loop {
            // handshake
            let handshake = self.rustls_client.is_handshaking();
            if !self.rustls_client.wants_write() && !self.rustls_client.wants_read() {
                return Ok((0, 0));
            };

            // writing
            while self.rustls_client.wants_write() {
                match self.rustls_client.write_tls(&mut self.buf_writer)? {
                    0 => {
                        self.buf_writer.flush()?;
                        return Ok((read, write));
                    }
                    n => {
                        write += n;
                    }
                };
            }
            self.buf_writer.flush()?;

            // If we are NOT handshaking, and written something,
            // return.
            if !handshake && write > 0 {
                return Ok((read, write));
            }

            // reading
            while !eof && self.rustls_client.wants_read() {
                let r = match self.rustls_client.read_tls(&mut self.buf_reader) {
                    Ok(0) => {
                        eof = true;
                        Some(0)
                    }
                    Ok(n) => {
                        read += n;
                        Some(n)
                    }
                    Err(e) if e.kind() == ErrorKind::Interrupted => {
                        debug!("ErrorKind: Interrupted");
                        None
                    }
                    Err(e) => return Err(e)?,
                };
                if r.is_some() {
                    break;
                }
            }

            // process new packets
            match self.rustls_client.process_new_packets() {
                Ok(iostate) => debug!("{:?}", iostate),
                Err(e) => return Err(e)?,
            }

            // If we are currently not handshaking, were handshaking in the past, and want to write.
            // loop back.
            if !self.rustls_client.is_handshaking() && handshake && self.rustls_client.wants_write()
            {
                continue;
            }

            // matching
            match (eof, handshake, self.rustls_client.is_handshaking()) {
                (_, true, false) => return Ok((read, write)),
                (_, false, _) => return Ok((read, write)),
                (true, true, true) => return Err("Shit went down.")?,
                (..) => {
                    dbg!((eof, handshake, self.rustls_client.is_handshaking()))
                }
            };
        }
    }
}
