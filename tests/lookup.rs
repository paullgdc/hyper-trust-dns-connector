use hyper::client::HttpConnector;
use hyper::Client;
use hyper_trust_dns_connector::{new_async_http_connector, AsyncHyperResolver};
use std::str::FromStr;

#[tokio::test]
async fn test_resolver_new_from_system_conf() {
    hyper_trust_dns_connector::AsyncHyperResolver::new_from_system_conf()
        .await
        .expect("couldn't create async resolver");
}

#[tokio::test]
async fn test_resolver_resolve() {
    use hyper::client::connect::dns::Name;
    use hyper::service::Service;

    let mut resolver = AsyncHyperResolver::new_from_system_conf()
        .await
        .expect("couldn't create async resolver");
    let _lookup_res = resolver
        .call(Name::from_str("google.com").unwrap())
        .await
        .expect("couldn't resolve google.com");
}

#[tokio::test]
async fn test_pub_new_async_http_connector() {
    let async_http = new_async_http_connector()
        .await
        .expect("couldn't create connector");
    let client: Client<HttpConnector<AsyncHyperResolver>> = Client::builder().build(async_http);
    client
        .get(hyper::Uri::from_static("http://httpbin.org/ip"))
        .await
        .expect("error during the request")
        .status();
}
