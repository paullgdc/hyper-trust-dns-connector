# hyper-trust-dns-connector

[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![crates.io](https://meritbadge.herokuapp.com/hyper-trust-dns-connector)](https://crates.io/crates/hyper-trust-dns-connector)
[![Released API docs](https://docs.rs/hyper-trust-dns-connector/badge.svg)](https://docs.rs/hyper-trust-dns-connector)

A crate to make [hickory-resolver](https://docs.rs/hickory-resolver/)'s (previously trust_dns_resolver)
asynchronous resolver compatible with [hyper](https://docs.rs/hyper) client,
to use instead of the default dns threadpool.

[Documentation](https://docs.rs/hyper-trust-dns-connector)

## Motivations

By default hyper HttpConnector uses the std provided resolver wich is blocking in a threadpool
with a configurable number of threads. This crate provides an alternative using hickory-resolver,
a dns resolver written in Rust, with async features.

## Example

```rust
use hyper::{Body, Client};
use hyper_trust_dns_connector::new_async_http_connector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let http = new_async_http_connector()?;
    let client = Client::builder().build::<_, Body>(http);
    let status_code = client
        .get(hyper::Uri::from_static("http://httpbin.org/ip"))
        .await?
        .status();
    assert_eq!(status_code, 200);
    Ok(())
}
```

## Contributing

If you need a feature implemented, or want to help, don't hesitate to open an issue or a PR.

## License

Provided under the MIT license ([LICENSE](LICENSE) or <http://opensource.org/licenses/MIT>)
