use crate::https::https_client::Client;
use crate::https::request::RequestBuilder;
mod https;

fn main() {
    let mut buf = Vec::new();
    let host = "www.rust-lang.org";
    let mut client = Client::new(host).unwrap();
    let req = RequestBuilder::new()
        .add_header("User-Agent: Haha/0.0.1")
        .add_header("Connection: close")
        .add_header("Accept-Encoding: identity")
        .set_method(https::request::HTTPMethods::GET)
        .set_host(host)
        .set_route("/")
        .set_content("".to_string())
        .build()
        .process()
        .unwrap();
    println!("{}", req);
    let wr = client.client_write(&req).unwrap();
    println!("Written: {} bytes", wr);
    let test = client.client_read(&mut buf).unwrap();
    println!("Read: {} bytes", test);
}
