use anyhow::{bail, ensure, Context as _, Result};
use url::Url;

use crate::hls_manifest_attributes::quoted_attribute;
use crate::hls_manifest_names::{attribute, tag};
use crate::hls_manifest_tags::{action, HlsTagAction};

pub use crate::hls_bootstrap::{inspect_hls_bootstrap, HlsBootstrap, UnsupportedHlsFeature};

pub const MAX_HLS_MANIFEST_BYTES: usize = 1024 * 1024;
pub const MAX_HLS_ASSET_BYTES: usize = 8 * 1024 * 1024;
const MAX_REWRITTEN_HLS_MANIFEST_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HlsResourceKind {
    Manifest,
    Asset,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HlsResource {
    pub url: Url,
    pub kind: HlsResourceKind,
}

/// Rewrites every resource in a bounded HLS manifest through `issue`.
///
/// # Errors
///
/// Returns an error when the manifest is malformed, unsupported, oversized, contains an unsafe
/// resource, or when `issue` cannot create a replacement URL.
pub fn rewrite_hls_manifest<Issue>(body: &[u8], base_url: &Url, mut issue: Issue) -> Result<String>
where
    Issue: FnMut(HlsResource) -> Result<String>,
{
    ensure!(
        body.len() <= MAX_HLS_MANIFEST_BYTES,
        "HLS manifest exceeds its byte limit"
    );
    let manifest = core::str::from_utf8(body).context("HLS manifest must be UTF-8")?;
    require_header(manifest)?;
    let mut rewritten = String::with_capacity(manifest.len());
    let mut next_uri_kind = None;
    for line in manifest.split_inclusive('\n') {
        rewrite_line(
            line,
            base_url,
            &mut issue,
            &mut next_uri_kind,
            &mut rewritten,
        )?;
        ensure!(
            rewritten.len() <= MAX_REWRITTEN_HLS_MANIFEST_BYTES,
            "rewritten HLS manifest exceeds its byte limit"
        );
    }
    if next_uri_kind.is_some() {
        bail!("HLS variant declaration is missing its URI");
    }
    Ok(rewritten)
}

fn require_header(manifest: &str) -> Result<()> {
    let first = manifest
        .trim_start_matches('\u{feff}')
        .lines()
        .next()
        .map(str::trim_end);
    if first != Some(tag::EXTM3U) {
        bail!("HLS manifest header is missing");
    }
    Ok(())
}

fn rewrite_line<Issue>(
    line: &str,
    base_url: &Url,
    issue: &mut Issue,
    next_uri_kind: &mut Option<HlsResourceKind>,
    output: &mut String,
) -> Result<()>
where
    Issue: FnMut(HlsResource) -> Result<String>,
{
    let content = line.trim_end_matches(['\r', '\n']);
    if content.is_empty() {
        output.push_str(line);
        return Ok(());
    }
    if content.starts_with('#') {
        let rewritten = rewrite_tag(content, base_url, issue, next_uri_kind)?;
        output.push_str(&rewritten);
        output.push_str(line_ending(line));
        return Ok(());
    }
    let url = resolve_http_url(base_url, content)?;
    let replacement = issue(HlsResource {
        url,
        kind: next_uri_kind.take().unwrap_or(HlsResourceKind::Asset),
    })?;
    output.push_str(&replacement);
    output.push_str(line_ending(line));
    Ok(())
}

fn rewrite_tag<Issue>(
    line: &str,
    base_url: &Url,
    issue: &mut Issue,
    next_uri_kind: &mut Option<HlsResourceKind>,
) -> Result<String>
where
    Issue: FnMut(HlsResource) -> Result<String>,
{
    match action(line)? {
        HlsTagAction::Pass => Ok(line.to_owned()),
        HlsTagAction::NextUri(kind) => {
            if next_uri_kind.replace(kind).is_some() {
                bail!("HLS variant declaration is missing its URI");
            }
            Ok(line.to_owned())
        }
        HlsTagAction::RewriteUri { kind, required } => {
            rewrite_uri_attribute(line, base_url, kind, required, issue)
        }
    }
}

fn rewrite_uri_attribute<Issue>(
    line: &str,
    base_url: &Url,
    kind: HlsResourceKind,
    required: bool,
    issue: &mut Issue,
) -> Result<String>
where
    Issue: FnMut(HlsResource) -> Result<String>,
{
    let span = quoted_attribute(line, attribute::URI)?;
    if required && span.is_none() {
        bail!("HLS tag is missing its URI attribute");
    }
    let Some((start, end)) = span else {
        return Ok(line.to_owned());
    };
    let url = resolve_http_url(base_url, &line[start..end])?;
    let replacement = issue(HlsResource { url, kind })?;
    Ok(format!("{}{}{}", &line[..start], replacement, &line[end..]))
}

fn resolve_http_url(base_url: &Url, reference: &str) -> Result<Url> {
    let url = base_url
        .join(reference)
        .context("resolve HLS resource URI")?;
    if !matches!(url.scheme(), "http" | "https") {
        bail!("HLS resource URI scheme is not allowed");
    }
    if !url.username().is_empty() || url.password().is_some() {
        bail!("HLS resource URI credentials are not allowed");
    }
    Ok(url)
}

fn line_ending(line: &str) -> &'static str {
    if line.ends_with("\r\n") {
        "\r\n"
    } else if line.ends_with('\n') {
        "\n"
    } else {
        ""
    }
}
