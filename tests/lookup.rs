extern crate futures;
extern crate hyper_trust_dns_connector;
extern crate hyper;
extern crate tokio;

use futures::Future;
use hyper_trust_dns_connector::{AsyncHyperResolver, new_async_http_connector};
use hyper::client::HttpConnector;
use std::str::FromStr;
use tokio::runtime::Runtime;

#[test]
fn test_resolver_new_from_system_conf() {
    hyper_trust_dns_connector::AsyncHyperResolver::new_from_system_conf()
        .expect("couldn't create async resolver");
}

#[test]
fn test_resolver_resolve() {
    use hyper::client::connect::dns::{Resolve, Name};
    let (resolver, background) = AsyncHyperResolver::new_from_system_conf()
        .expect("couldn't create async resolver");
    let mut rt = Runtime::new().expect("couldn't create runtime");
    rt.spawn(background);
    let _lookup_res = rt.block_on(resolver.resolve(Name::from_str("google.com").unwrap()))
        .expect("couldn't resolve google.com");
}

#[test]
fn test_pub_new_async_http_connector() {
    let mut rt = Runtime::new().expect("couldn't create runtime");
    let (async_http, background) = new_async_http_connector()
        .expect("couldn't create connector");
    let client : hyper::Client<HttpConnector<AsyncHyperResolver>> = hyper::Client::builder()
        .executor(rt.executor())
        .build(async_http);
    rt.spawn(background);
    rt.block_on(client.get(hyper::Uri::from_static("http://httpbin.org/ip")).map(|res| res.status()))
        .expect("couldn't use the client");
}
