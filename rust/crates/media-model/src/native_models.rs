#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum NativeVideoCacheKey {
    AdvertisedDigest(String),
    UrlDerived(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeVideoDelivery {
    Progressive,
    Hls,
}
