#[test]
fn test_readme_example() {
    extern crate hyper_trust_dns_connector;
    extern crate hyper;
    extern crate tokio;

    use hyper_trust_dns_connector::new_async_http_connector;
    use hyper::{Client, Body};

    #[tokio::main]
    async fn main() {
        let http = new_async_http_connector()
            .await
            .expect("couldn't create connector");
        let client = Client::builder()
            .build::<_, Body>(http);
        let status_code = client.get(hyper::Uri::from_static("http://httpbin.org/ip"))
            .await
            .expect("error during the request")
            .status();
        println!("status is {:?}", status_code);
    }
    main();
}
