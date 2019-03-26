# hyper-trust-dns-connector

A compatibility crate to use trust-dns-resolver asynchronously with hyper client, instead the default dns threadpool

## Motivations

By default hyper HttpConnector uses the std provided resolver wich is blocking in a threadpool with a configurable number of threads.
This crate provides an alternative using trust_dns_resolver, a dns resolver written in Rust, with async features.

## Example

```rust
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
    let status_code = rt
        .block_on(client.get(hyper::Uri::from_static("http://httpbin.org/ip"))
        .map(|res| res.status()))
        .expect("error during the request");
    println!("status is {:?}", status_code);
}
```
