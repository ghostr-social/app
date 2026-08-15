use crate::hls_manifest::MAX_HLS_MANIFEST_BYTES;
use crate::hls_manifest_attributes::quoted_attribute;
use anyhow::{bail, ensure, Context, Result};
use url::Url;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HlsBootstrap {
    Master { variant: Url },
    Media { init: Option<Url>, segment: Url },
}

pub fn inspect_hls_bootstrap(body: &[u8], base_url: &Url) -> Result<HlsBootstrap> {
    ensure!(
        body.len() <= MAX_HLS_MANIFEST_BYTES,
        "HLS manifest exceeds its byte limit"
    );
    let text = std::str::from_utf8(body).context("HLS manifest must be UTF-8")?;
    require_header(text)?;
    let mut scan = Scan::default();
    for raw in text.lines().skip(1) {
        scan.visit(raw.trim(), base_url)?;
        if let Some(variant) = scan.variant.take() {
            return Ok(HlsBootstrap::Master { variant });
        }
    }
    scan.finish()
}

#[derive(Default)]
struct Scan {
    awaiting_variant: bool,
    saw_extinf: bool,
    end_list: bool,
    init: Option<Url>,
    segment: Option<Url>,
    variant: Option<Url>,
}

impl Scan {
    fn visit(&mut self, line: &str, base_url: &Url) -> Result<()> {
        if line.is_empty() {
            return Ok(());
        }
        if forbidden(line) {
            bail!("unsupported HLS playlist feature");
        }
        if self.awaiting_variant && line.starts_with('#') {
            bail!("HLS variant declaration is missing its URI");
        }
        if line.starts_with("#EXT-X-STREAM-INF:") {
            self.awaiting_variant = true;
            return Ok(());
        }
        if line.starts_with("#EXT-X-MAP:") {
            self.init = Some(attribute_url(line, base_url)?);
            return Ok(());
        }
        if line.starts_with("#EXTINF:") {
            self.saw_extinf = true;
            return Ok(());
        }
        if line == "#EXT-X-ENDLIST" {
            self.end_list = true;
            return Ok(());
        }
        if line.starts_with('#') {
            return Ok(());
        }
        let url = resolve(base_url, line)?;
        if self.awaiting_variant {
            self.awaiting_variant = false;
            self.variant = Some(url);
        } else if self.saw_extinf && self.segment.is_none() {
            self.segment = Some(url);
            self.saw_extinf = false;
        }
        Ok(())
    }

    fn finish(self) -> Result<HlsBootstrap> {
        ensure!(
            !self.awaiting_variant,
            "HLS variant declaration is missing its URI"
        );
        ensure!(self.end_list, "live HLS playlists are not prefetched");
        let segment = self
            .segment
            .ok_or_else(|| anyhow::anyhow!("HLS media playlist has no playable segment"))?;
        Ok(HlsBootstrap::Media {
            init: self.init,
            segment,
        })
    }
}

fn require_header(text: &str) -> Result<()> {
    let first = text
        .trim_start_matches('\u{feff}')
        .lines()
        .next()
        .map(str::trim);
    ensure!(first == Some("#EXTM3U"), "HLS manifest header is missing");
    Ok(())
}

fn attribute_url(line: &str, base_url: &Url) -> Result<Url> {
    let (start, end) = quoted_attribute(line, "URI")?
        .ok_or_else(|| anyhow::anyhow!("HLS map is missing its URI"))?;
    resolve(base_url, &line[start..end])
}

fn resolve(base_url: &Url, reference: &str) -> Result<Url> {
    let url = base_url
        .join(reference)
        .context("resolve HLS bootstrap URI")?;
    ensure!(
        matches!(url.scheme(), "http" | "https"),
        "HLS URI scheme is not allowed"
    );
    ensure!(
        url.username().is_empty() && url.password().is_none(),
        "HLS URI credentials are not allowed"
    );
    Ok(url)
}

fn forbidden(line: &str) -> bool {
    [
        "#EXT-X-KEY:",
        "#EXT-X-SESSION-KEY:",
        "#EXT-X-BYTERANGE:",
        "#EXT-X-PART:",
        "#EXT-X-PART-INF:",
        "#EXT-X-SERVER-CONTROL:",
        "#EXT-X-SKIP:",
        "#EXT-X-PRELOAD-HINT:",
        "#EXT-X-RENDITION-REPORT:",
    ]
    .iter()
    .any(|prefix| line.starts_with(prefix))
}
