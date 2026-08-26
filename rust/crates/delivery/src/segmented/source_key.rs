use std::collections::HashSet;
use url::Url;

pub(crate) fn canonical(raw: &str) -> String {
    let Ok(mut parsed) = Url::parse(raw) else {
        return raw.to_owned();
    };
    parsed.set_fragment(None);
    parsed.to_string()
}

pub(super) fn contains(sources: &[String], source: &str) -> bool {
    let expected = canonical(source);
    sources.iter().any(|known| canonical(known) == expected)
}

pub(super) fn same_members(left: &[String], right: &[String]) -> bool {
    keys(left) == keys(right)
}

fn keys(sources: &[String]) -> HashSet<String> {
    sources.iter().map(|source| canonical(source)).collect()
}
