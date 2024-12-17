use crate::https_client::Client;
mod https_client;

fn main() {
    let mut buf = Vec::new();
    let mut client = Client::new("www.rust-lang.org").unwrap();
    let wr = client
        .client_write(concat!(
            "GET / HTTP/1.1\r\n,",
            "Host: www.rust-lang.org\r\n",
            "Connection: close\r\n",
            "Accept-Encoding: identity\r\n",
            "\r\n"
        ))
        .unwrap();

    println!("Written: {} bytes", wr);
    let test = client.client_read(&mut buf).unwrap();
    println!("Read: {} bytes", test);
    println!("{}", std::str::from_utf8(&buf).unwrap());
}
