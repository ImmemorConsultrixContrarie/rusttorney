use std::borrow::Cow;
use futures::future::BoxFuture;

pub trait Command<'a> {
    fn from_protocol(name: &'a str, args: impl Iterator<Item = &'a str>) -> Result<Self, anyhow::Error> where Self: Sized;

    fn handle(&self) -> BoxFuture<'static, ()>;
}