use super::request::RequestBuilder;
use crate::https::client::Methods;
use crate::https::url::UrlError;
// Defines if a type can be a HTTPS client
// which is accepted by the `RequestBuilder` as an executor of the `Request`
pub trait CanBeClient<'a> {
    fn request(&'a mut self, m: Methods, url: &'a str) -> Result<RequestBuilder<'a>, UrlError>;
}
