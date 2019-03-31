#[cfg(feature="hyper-tls-connector")]
mod tests {
    use tokio::runtime::Runtime;
    use hyper_trust_dns_connector::https::new_async_https_connector;
    use hyper::{Client, Body};

    #[test]
    fn test_https_connector() {
        let mut rt = Runtime::new().expect("couldn't create runtime");

        let (https, background) = new_async_https_connector()
            .expect("couldn't create connector");
        let client = Client::builder()
            .executor(rt.executor())
            .build::<_, Body>(https);

        rt.spawn(background);
        let status_code = rt.block_on(client.get(hyper::Uri::from_static("https://httpbin.org/ip")))
            .map(|res| res.status())
            .expect("error during the request");
        println!("status is {:?}", status_code);
    }
}