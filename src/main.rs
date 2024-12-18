use crate::https::https_client::Client;
use crate::https::request::RequestBuilder;
use tokio::spawn;
mod https;

#[tokio::main]
async fn main() {
    let mut buf: Vec<u8> = vec![];
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
    let wr = client.client_write(&req);
    wr.await.unwrap();
    println!("Write!");

    let test = client.receive(&mut buf);
    println!("Reading!");
    println!("Read!");
    println!("{:?}", test.await.unwrap())
}
