extern crate hyper;
extern crate hyper_trust_dns_connector;
extern crate tokio;

use hyper::client::HttpConnector;
use hyper_trust_dns_connector::{new_async_http_connector, AsyncHyperResolver};
use std::str::FromStr;
use tokio::runtime::Runtime;

#[test]
fn test_resolver_new_from_system_conf() {
    let mut rt = Runtime::new().expect("couldn't create runtime");
    rt.block_on(hyper_trust_dns_connector::AsyncHyperResolver::new_from_system_conf())
        .expect("couldn't create async resolver");
}

#[test]
fn test_resolver_resolve() {
    use hyper::client::connect::dns::Name;
    use hyper::service::Service;

    let mut rt = Runtime::new().expect("couldn't create runtime");
    rt.block_on(async {
        let mut resolver = AsyncHyperResolver::new_from_system_conf()
            .await
            .expect("couldn't create async resolver");
        let _lookup_res = resolver.call(Name::from_str("google.com").unwrap()).await
            .expect("couldn't resolve google.com");
    });
}

#[test]
fn test_pub_new_async_http_connector() {
    let mut rt = Runtime::new().expect("couldn't create runtime");
    let async_http = rt.block_on(async {
        new_async_http_connector().await.expect("couldn't create connector")
    });
    let client: hyper::Client<HttpConnector<AsyncHyperResolver>> = hyper::Client::builder()
        .build(async_http);
    rt.block_on( async {
        client
            .get(hyper::Uri::from_static("http://httpbin.org/ip"))
            .await.expect("error during the request")
            .status()
    });
}
