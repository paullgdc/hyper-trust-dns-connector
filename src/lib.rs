#![cfg_attr(docsrs, feature(doc_cfg))]
//! # hyper_trust_dns_connector
//!
//! A crate to make [trust-dns-resolver](https://docs.rs/trust-dns-resolver)'s
//! asynchronous resolver compatible with [hyper](https://docs.rs/hyper) client,
//! to use instead of the default dns threadpool.
//!
//! ## Features
//!
//!  * `hyper-tls-connector` This feature includes
//! [`hyper-tls`](https://docs.rs/hyper-tls/0.5/hyper_tls/) and
//! [`native-tls`](https://docs.rs/native-tls/0.2/native_tls/) to
//!     provide a helper function to create a tls connector.
//!
//! ## Usage
//!
//! [trust-dns-resolver](https://docs.rs/trust-dns-resolver) creates an async resolver
//! for dns queries, which is then used by hyper
//!
//! ## Example
//!
//! ```
//! use hyper_trust_dns_connector::new_async_http_connector;
//! use hyper::{Client, Body};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let http = new_async_http_connector()?;
//!     let client = Client::builder().build::<_, Body>(http);
//!     let res = client.get(hyper::Uri::from_static("http://httpbin.org/ip"))
//!         .await?;
//!     assert_eq!(res.status(), 200);
//!     Ok(())
//! }
//! ```

use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::TokioAsyncResolver;
use hyper::client::connect::dns::Name;
use hyper::client::HttpConnector;
use hyper::service::Service;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{future::Future, net::SocketAddr, net::ToSocketAddrs};

/// Wrapper around trust-dns-resolver's
/// [`TokioAsyncResolver`](https://docs.rs/trust-dns-resolver/0.20.0/trust_dns_resolver/type.TokioAsyncResolver.html)
///
/// The resolver runs a background Task which manages dns requests. When a new resolver is created,
/// the background task is also created, it needs to be spawned on top of an executor before using the client,
/// or dns requests will block.
#[derive(Debug, Clone)]
pub struct AsyncHyperResolver(TokioAsyncResolver);

impl AsyncHyperResolver {
    /// constructs a new resolver, arguments are passed to the corresponding method of
    /// [`TokioAsyncResolver`](https://docs.rs/trust-dns-resolver/0.20.0/trust_dns_resolver/type.TokioAsyncResolver.html#method.new)
    pub fn new(config: ResolverConfig, options: ResolverOpts) -> Result<Self, io::Error> {
        let resolver = TokioAsyncResolver::tokio(config, options);
        Ok(Self(resolver))
    }

    /// constructs a new resolver from default configuration, uses the corresponding method of
    /// [`TokioAsyncResolver`](https://docs.rs/trust-dns-resolver/0.20.0/trust_dns_resolver/type.TokioAsyncResolver.html#method.new)
    pub fn new_from_system_conf() -> Result<Self, io::Error> {
        let resolver = TokioAsyncResolver::tokio_from_system_conf()?;
        Ok(Self(resolver))
    }
}

impl Service<Name> for AsyncHyperResolver {
    type Response = std::vec::IntoIter<SocketAddr>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
    type Error = io::Error;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, name: Name) -> Self::Future {
        let resolver = self.0.clone();
        Box::pin((|| async move {
            Ok(resolver
                .lookup_ip(name.as_str())
                .await?
                .iter()
                .map(|addr| (addr, 0_u16).to_socket_addrs())
                .try_fold(Vec::new(), |mut acc, s_addr| {
                    acc.extend(s_addr?);
                    Ok::<_, io::Error>(acc)
                })?
                .into_iter())
        })())
    }
}

/// A helper function to create an http connector and a dns task with the default configuration
///
/// ```
/// use hyper_trust_dns_connector::new_async_http_connector;
/// use hyper::{Client, Body};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let async_http = new_async_http_connector()?;
/// let client = Client::builder().build::<_, Body>(async_http);
/// # Ok(())
/// # }
/// ```
pub fn new_async_http_connector() -> Result<HttpConnector<AsyncHyperResolver>, io::Error> {
    let resolver = AsyncHyperResolver::new_from_system_conf()?;
    Ok(HttpConnector::new_with_resolver(resolver))
}

/// Provides a helper method to create an https connector using
/// [`hyper-tls`](https://docs.rs/hyper-tls/0.5/hyper_tls/)
///
/// ## Example
///
/// ```
/// use hyper::Client;
/// use hyper_trust_dns_connector::https::new_async_https_connector;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let https_connector = new_async_https_connector()?;
///     let client: Client<_> = Client::builder().build(https_connector);
///     let res = client
///         .get(hyper::Uri::from_static("https://httpbin.org/ip"))
///         .await?;
///     assert_eq!(res.status(), 200);
///     Ok(())
/// }
/// ```
#[cfg(feature = "hyper-tls-connector")]
#[cfg_attr(docsrs, doc(cfg(feature = "hyper-tls-connector")))]
pub mod https {

    use hyper_tls::HttpsConnector;
    use native_tls::TlsConnector;

    use crate::io;
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

    /// A helper function to create an https connector from [`hyper-tls`](https://docs.rs/hyper-tls/0.5/hyper_tls/)
    /// and a dns task with the default configuration.
    pub fn new_async_https_connector(
    ) -> Result<HttpsConnector<HttpConnector<AsyncHyperResolver>>, Error> {
        let mut http = new_async_http_connector()?;
        http.enforce_http(false);
        let tls_connector = TlsConnector::new()?;
        Ok(HttpsConnector::from((http, tls_connector.into())))
    }
}
