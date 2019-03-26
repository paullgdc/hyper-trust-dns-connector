#[test]
fn test_readme_example() {
    extern crate hyper_trust_dns_connector;
    extern crate hyper;
    extern crate tokio;

    use hyper_trust_dns_connector::new_async_http_connector;
    use hyper::{Client, Body};
    use tokio::prelude::Future;
    use tokio::runtime::Runtime;

    fn main() {
        let mut rt = Runtime::new().expect("couldn't create runtime");
        let (async_http, background) = new_async_http_connector()
            .expect("couldn't create connector");
        let client = Client::builder()
            .executor(rt.executor())
            .build::<_, Body>(async_http);
        rt.spawn(background);
        let status_code = rt.block_on(client.get(hyper::Uri::from_static("http://httpbin.org/ip")).map(|res| res.status()))
            .expect("error during the request");
        println!("status is {:?}", status_code);
    }
    main();

}
