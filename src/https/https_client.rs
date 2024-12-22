use crate::https::parse_http::parse_http;
use crate::https::response::Response;
use rustls::ClientConfig;
use rustls::ClientConnection;
use rustls::RootCertStore;
use rustls::DEFAULT_VERSIONS;
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
    pub fn new(server: &str) -> Result<Self, Box<dyn std::error::Error>> {
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
        let mut conn = match ClientConnection::new(Arc::new(cfg), dns_name) {
            Ok(c) => c,
            Err(error) => {
                panic!("Failed to create a TLS Client Connection, error: {}", error)
            }
        };

        let mut tcp_stream = TcpStream::connect(server.to_string() + ":443").unwrap();
        Ok(Self {
            rustls_client: conn,
            server_name: server.to_owned(),
            buf_reader: BufReader::new(TcpStream::try_clone(&tcp_stream)?),
            buf_writer: BufWriter::new(TcpStream::try_clone(&tcp_stream)?),
            tcp_stream,
        })
    }

    // Writes to the connection
    pub fn client_write(&mut self, stuff: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let written;

        match self.rustls_client.writer().write(stuff.as_bytes()) {
            Err(error) => return Err(error)?,
            Ok(n) => written = n,
        }

        let (r, w) = self
            .rustls_client
            .complete_io(&mut self.tcp_stream)
            .unwrap();
        println!("WRITE R: {0}, W: {1}", r, w);
        Ok(written)
    }

    pub fn client_read(&mut self, o: &mut Vec<u8>) -> Result<usize, Box<dyn std::error::Error>> {
        self.rustls_first_io()?;

        while self.rustls_client.wants_read() {
            let len = self.rustls_client.read_tls(&mut self.buf_reader)?;
            if len == 0 {
                break;
            }
        }
        self.rustls_client.process_new_packets()?;
        Ok(self.rustls_client.reader().read_to_end(o)?)
    }
    fn rustls_first_io(&mut self) -> Result<(), std::io::Error> {
        if self.rustls_client.is_handshaking() {
            self.rustls_client.complete_io(&mut self.tcp_stream)?;
        };
        if self.rustls_client.wants_write() {
            self.rustls_client.complete_io(&mut self.tcp_stream)?;
        };
        Ok(())
    }
}
