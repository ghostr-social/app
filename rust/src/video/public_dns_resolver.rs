use crate::video::native_cache_failure::permanent_cause;
use crate::video::public_media_address::is_public;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use std::error::Error;
use std::io;
use std::sync::Arc;
use tokio::net::lookup_host;

type ResolveError = Box<dyn Error + Send + Sync>;

pub struct SystemResolver;

impl Resolve for SystemResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let host = name.as_str().to_owned();
        Box::pin(async move {
            let addresses = lookup_host((host, 0)).await.map_err(resolve_error)?;
            Ok(Box::new(addresses) as Addrs)
        })
    }
}

pub struct PublicDnsResolver {
    inner: Arc<dyn Resolve>,
}

impl PublicDnsResolver {
    pub fn new<R: Resolve + 'static>(inner: Arc<R>) -> Self {
        Self { inner }
    }
}

impl Resolve for PublicDnsResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let resolving = self.inner.resolve(name);
        Box::pin(async move { public_addresses(resolving.await?).map_err(resolve_error) })
    }
}

fn public_addresses(addresses: Addrs) -> io::Result<Addrs> {
    let public = addresses
        .filter(|address| is_public(address.ip()))
        .collect::<Vec<_>>();
    if public.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            permanent_cause("media host has no public address"),
        ));
    }
    Ok(Box::new(public.into_iter()))
}

fn resolve_error(error: impl Error + Send + Sync + 'static) -> ResolveError {
    Box::new(error)
}
