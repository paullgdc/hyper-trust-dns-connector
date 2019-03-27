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
/// [`AsyncResolver`](../../10.3/trust_dns_resolver/struct.AsyncResolver.html)
#[derive(Debug, Clone)]
pub struct AsyncHyperResolver(AsyncResolver);

impl AsyncHyperResolver {
    /// constructs a new resolver, arguments are passed to
    /// https://docs.rs/trust-dns-resolver/0.10.3/trust_dns_resolver/struct.AsyncResolver.html#method.new 
    pub fn new(
        config: ResolverConfig,
        options: ResolverOpts,
    ) -> (Self, impl Future<Item = (), Error = ()>) {
        let (resolver, background) = AsyncResolver::new(config, options);
        (Self(resolver), background)
    }
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
