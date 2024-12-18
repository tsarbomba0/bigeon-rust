use crate::https::parse_http::parse_http;
use crate::https::response::Response;
use rustls::ClientConfig;
use rustls::ClientConnection;
use rustls::RootCertStore;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

// Client struct
pub struct Client {
    server_name: String,
    rustls_client: ClientConnection,
    buf_reader: BufReader<TcpStream>,
    buf_writer: BufWriter<TcpStream>,
    tcp_stream: TcpStream,
}

// Client functions.
impl Client {
    pub fn new(server: &str) -> Result<Self, std::io::Error> {
        let cfg = ClientConfig::builder()
            .with_root_certificates(RootCertStore {
                roots: webpki_roots::TLS_SERVER_ROOTS.into(),
            })
            .with_no_client_auth();

        let server_name = String::from(server);
        let dns_name = match server_name.try_into() {
            Ok(name) => {
                println!("Connecting to: {:?}", name);
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
        //tcp_stream.set_nonblocking(true)?;

        Ok(Self {
            rustls_client: conn,
            server_name: server.to_owned(),
            buf_reader: BufReader::new(TcpStream::try_clone(&tcp_stream)?),
            buf_writer: BufWriter::new(TcpStream::try_clone(&tcp_stream)?),
            tcp_stream,
        })
    }

    // Writes to the connection
    pub async fn client_write(&mut self, stuff: &str) -> Result<usize, std::io::Error> {
        let written: usize;
        loop {
            if self.rustls_client.wants_write() {
                let mut buf = Vec::new();
                match self.rustls_client.writer().write(stuff.as_bytes()) {
                    Err(error) => return Err(error),
                    Ok(n) => written = n,
                }
                self.rustls_client.write_tls(&mut buf)?;
                match self.buf_writer.write_all(&buf) {
                    Ok(_) => (),
                    Err(error) => return Err(error),
                }
                self.buf_writer.flush()?;
                break;
            };
        }
        Ok(written)
    }

    // Reads from the connection
    pub fn client_read(&mut self, output: &mut Vec<u8>) -> Result<usize, std::io::Error> {
        let mut len = 0;
        loop {
            if self.rustls_client.wants_read() {
                self.rustls_client
                    .complete_io(&mut self.tcp_stream)
                    .unwrap();
                match self.rustls_client.read_tls(&mut self.buf_reader) {
                    Ok(_) => {
                        self.rustls_client.process_new_packets().unwrap();
                    }
                    Err(error) => {
                        if error.kind() != std::io::ErrorKind::WouldBlock {
                            return Err(error);
                        }
                    }
                }
            }
            len = match self.rustls_client.reader().read_to_end(output) {
                Ok(n) => {
                    len += n;
                    break;
                }
                Err(error) => {
                    if error.kind() != std::io::ErrorKind::WouldBlock {
                        return Err(error);
                    } else {
                        0
                    }
                }
            }
        }
        Ok(len)
    }

    pub async fn receive(
        &mut self,
        buf: &mut Vec<u8>,
    ) -> Result<Response, Box<dyn std::error::Error>> {
        self.client_read(buf)?;
        parse_http(buf)
    }
}
