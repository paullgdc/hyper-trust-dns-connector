#[test]
fn test_readme_example() -> Result<(), Box<dyn std::error::Error>> {
    use hyper::{Body, client::Client};
    use hyper_trust_dns_connector::new_async_http_connector;

    #[tokio::main]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {
        let http = new_async_http_connector().await?;
        let client = Client::builder().build::<_, Body>(http);
        let status_code = client
            .get(hyper::Uri::from_static("http://httpbin.org/ip"))
            .await?
            .status();
        assert_eq!(status_code, 200);
        Ok(())
    }
    main()
}
