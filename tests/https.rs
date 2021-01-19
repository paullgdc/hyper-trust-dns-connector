#[cfg(feature = "hyper-tls-connector")]
mod tests {
    use hyper::{body::to_bytes, Client};
    use hyper_trust_dns_connector::https::new_async_https_connector;

    #[tokio::test]
    async fn test_https_connector() {
        let async_https = new_async_https_connector().expect("couldn't create connector");
        let client: Client<_> = Client::builder().build(async_https);
        let mut res = client
            .get(hyper::Uri::from_static("https://httpbin.org/ip"))
            .await
            .expect("error during the request");
        let status_code = res.status();
        println!("status is {:?}", status_code);
        println!("ip is {:?}", to_bytes(res.body_mut()).await.unwrap())
    }
}
