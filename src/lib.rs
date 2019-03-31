#![doc(html_root_url = "https://docs.rs/hyper/0.12.25")]
//! # hyper_trust_dns_connector
//! 
//! A compatibility crate to use [trust-dns-resolver](https://docs.rs/trust-dns-resolver) 
//! asynchronously with [hyper](https://docs.rs/hyper) client,
//! instead the default dns threadpool.
//! 
//! ## Features
//!
//!  * `hyper-tls-connector` This feature includes
//! [`hyper-tls`](https://docs.rs/hyper-tls/0.3/hyper_tls/) and
//! [`native-tls`](https://docs.rs/native-tls/0.2/native_tls/) to
//!     provide a helper function to create a tls connector.
//!
//! ## Example
//!
//! ```
//! extern crate hyper_trust_dns_connector;
//! extern crate hyper;
//! extern crate tokio;
//! 
//! use hyper_trust_dns_connector::new_async_http_connector;
//! use hyper::{Client, Body};
//! use tokio::prelude::Future;
//! use tokio::runtime::Runtime;
//! 
//! let mut rt = Runtime::new().expect("couldn't create runtime");
//! let (async_http, background) = new_async_http_connector()
//!     .expect("couldn't create connector");
//! let client = Client::builder()
//!     .executor(rt.executor())
//!     .build::<_, Body>(async_http);
//! rt.spawn(background);
//! let status_code = rt
//!     .block_on(client.get(hyper::Uri::from_static("http://httpbin.org/ip"))
//!     .map(|res| res.status()))
//!     .expect("error during the request");
//! println!("status is {:?}", status_code);
//! ```
extern crate futures;
extern crate hyper;
extern crate trust_dns_resolver;

use futures::{Async, Future, Poll};
use hyper::client::connect::dns::{Name, Resolve};
use hyper::client::HttpConnector;
use std::io;
use std::net::IpAddr;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::{AsyncResolver, BackgroundLookupIp};

/// Wrapper future type around trust-dns-resolver's 
/// [`BackgroundLookupIp`](https://docs.rs/trust-dns-resolver/0.10.3/trust_dns_resolver/type.BackgroundLookupIp.html)
pub struct HyperLookupFuture(BackgroundLookupIp);

impl Future for HyperLookupFuture {
    type Item = std::vec::IntoIter<IpAddr>;
    type Error = io::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let lookups = self.0.poll()?;
        Ok(match lookups {
            Async::NotReady => Async::NotReady,
            Async::Ready(lookups) => {
                Async::Ready(lookups.iter().collect::<Vec<IpAddr>>().into_iter())
            }
        })
    }
}
/// Wrapper type around trust-dns-resolver's 
/// [`AsyncResolver`](https://docs.rs/trust-dns-resolver/0.10.3/trust_dns_resolver/struct.AsyncResolver.html)
/// 
/// The resolver runs a bakground Task wich manages dns requests. When a new resolver is created, 
/// the background task is also created, it needs to be spawned on top of an executor before using the client,
/// or dns requests will block.
#[derive(Debug, Clone)]
pub struct AsyncHyperResolver(AsyncResolver);

impl AsyncHyperResolver {
    /// constructs a new resolver, arguments are passed to the corresponding method of
    /// [`AsyncResolver`](https://docs.rs/trust-dns-resolver/0.10.3/trust_dns_resolver/struct.AsyncResolver.html#method.new)
    pub fn new(
        config: ResolverConfig,
        options: ResolverOpts,
    ) -> (Self, impl Future<Item = (), Error = ()>) {
        let (resolver, background) = AsyncResolver::new(config, options);
        (Self(resolver), background)
    }
    /// constructs a new resolver from default configuration, uses the corresponding method of
    /// [`AsyncResolver`](https://docs.rs/trust-dns-resolver/0.10.3/trust_dns_resolver/struct.AsyncResolver.html#method.new) 
    pub fn new_from_system_conf() -> Result<(Self, impl Future<Item = (), Error = ()>), io::Error> {
        let (resolver, background) = AsyncResolver::from_system_conf()?;
        Ok((Self(resolver), background))
    }
}

impl Resolve for AsyncHyperResolver {
    type Addrs = std::vec::IntoIter<IpAddr>;
    type Future = HyperLookupFuture;
    fn resolve(&self, name: Name) -> Self::Future {
        HyperLookupFuture(self.0.lookup_ip(name.as_str()))
    }
}

/// A helper function to create an http connector from default configuration
/// 
/// ```
/// use tokio::runtime::Runtime;
/// use hyper_trust_dns_connector::new_async_http_connector;
/// use hyper::{Client, Body};
/// 
/// let mut rt = Runtime::new().expect("couldn't create runtime");
/// 
/// let (async_http, background) = new_async_http_connector()
///     .expect("couldn't create connector");
/// let client = Client::builder()
///     .executor(rt.executor())
///     .build::<_, Body>(async_http);
/// 
/// rt.spawn(background);
/// ```
pub fn new_async_http_connector() -> Result<
    (
        HttpConnector<AsyncHyperResolver>,
        impl Future<Item = (), Error = ()>,
    ),
    io::Error,
> {
    let (resolver, background) = AsyncHyperResolver::new_from_system_conf()?;
    Ok((HttpConnector::new_with_resolver(resolver), background))
}

/// Module to use [`hyper-tls`](https://docs.rs/hyper-tls/0.3/hyper_tls/),
/// needs "hyper-tls-connector" feature enabled
/// 
/// ## Example 
/// 
/// ```
/// use tokio::runtime::Runtime;
/// use hyper_trust_dns_connector::https::new_async_https_connector;
/// use hyper::{Client, Body};
///
/// let mut rt = Runtime::new().expect("couldn't create runtime");
///
/// let (async_https, background) = new_async_https_connector()
///     .expect("couldn't create connector");
/// let client = Client::builder()
///     .executor(rt.executor())
///     .build::<_, Body>(async_https);
///
/// rt.spawn(background);
/// ```
#[cfg(feature = "hyper-tls-connector")]
pub mod https {

    extern crate hyper_tls;
    extern crate native_tls;

    use hyper_tls::HttpsConnector;
    use native_tls::TlsConnector;

    use crate::io;
    use crate::Future;
    use crate::HttpConnector;
    use crate::{new_async_http_connector, AsyncHyperResolver};

    #[derive(Debug)]
    pub enum Error {
        NativeTls(native_tls::Error),
        Io(io::Error),
    }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                Error::NativeTls(err) => write!(f, "native_tls error : {}", err),
                Error::Io(err) => write!(f, "io error : {}", err),
            }
        }
    }

    impl std::error::Error for Error {}
    impl From<io::Error> for Error {
        fn from(error: io::Error) -> Self {
            Error::Io(error)
        }
    }

    impl From<native_tls::Error> for Error {
        fn from(error: native_tls::Error) -> Self {
            Error::NativeTls(error)
        }
    }

    /// A helper function to create an https connector from [`hyper-tls`](https://docs.rs/hyper-tls/0.3/hyper_tls/)
    /// from default configuration.
    pub fn new_async_https_connector() -> Result<
        (
            HttpsConnector<HttpConnector<AsyncHyperResolver>>,
            impl Future<Item = (), Error = ()>,
        ),
        Error,
    > {
        let (mut http, background) = new_async_http_connector()?;
        http.enforce_http(false);
        let tls_connector = TlsConnector::new()?;
        Ok((HttpsConnector::from((http, tls_connector)), background))
    }

}
