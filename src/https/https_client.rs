//use crate::https::response::Response;
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
    pub(crate) server_name: String,
    rustls_client: ClientConnection,
    buf_reader: BufReader<TcpStream>,
    #[allow(dead_code)]
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
    pub fn client_write(&mut self, stuff: &[u8]) -> Result<usize, Box<dyn std::error::Error>> {
        self.rustls_first_io()?;

        let written = match self.rustls_client.writer().write(stuff) {
            Err(error) => return Err(error)?,
            Ok(n) => n,
        };

        self.rustls_client.complete_io(&mut self.tcp_stream)?;
        Ok(written)
    }

    pub fn client_read(&mut self, o: &mut Vec<u8>) -> Result<usize, Box<dyn std::error::Error>> {
        self.rustls_first_io()?;
        while self.rustls_client.wants_read() {
            let len = self.rustls_client.read_tls(&mut self.buf_reader)?;
            self.rustls_client.process_new_packets()?;
            if len == 0 {
                break;
            }
        }
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
