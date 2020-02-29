#[cfg(feature = "hyper-tls-connector")]
mod tests {
    use hyper::Client;
    use hyper_trust_dns_connector::https::new_async_https_connector;
    use tokio::runtime::Runtime;

    #[test]
    fn test_https_connector() {
        let mut rt = Runtime::new().expect("couldn't create runtime");
        let async_https = rt.block_on(async {
            new_async_https_connector()
                .await
                .expect("couldn't create connector")
        });
        let client: Client<_> = Client::builder().build(async_https);
        let status_code = rt.block_on(async {
            client
                .get(hyper::Uri::from_static("https://httpbin.org/ip"))
                .await
                .expect("error during the request")
                .status()
        });
        println!("status is {:?}", status_code);
    }
}
