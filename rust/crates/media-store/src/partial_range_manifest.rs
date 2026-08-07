use anyhow::{bail, Result};
use std::ops::Range;

/// Normalized set of present byte ranges for one partially downloaded video.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RangeManifest {
    total_len: Option<u64>,
    ranges: Vec<(u64, u64)>,
}

impl RangeManifest {
    pub fn total_len(&self) -> Option<u64> {
        self.total_len
    }

    pub fn set_total_len(&mut self, len: u64) -> Result<()> {
        if self.total_len.is_some_and(|current| current != len) {
            bail!("total length is already declared with a different value");
        }
        if self.ranges.last().is_some_and(|&(_, end)| end > len) {
            bail!("total length is shorter than the stored bytes");
        }
        self.total_len = Some(len);
        Ok(())
    }

    pub fn insert(&mut self, span: Range<u64>) -> Result<()> {
        if span.start >= span.end {
            return Ok(());
        }
        if self.total_len.is_some_and(|len| span.end > len) {
            bail!("range extends past the declared total length");
        }
        self.ranges.push((span.start, span.end));
        self.ranges.sort_unstable();
        self.ranges = coalesce(&self.ranges);
        Ok(())
    }

    pub fn ranges(&self) -> Vec<Range<u64>> {
        self.ranges.iter().map(|&(start, end)| start..end).collect()
    }

    pub fn covered_bytes(&self) -> u64 {
        self.ranges.iter().map(|(start, end)| end - start).sum()
    }

    pub fn contains(&self, span: &Range<u64>) -> bool {
        span.start >= span.end
            || self
                .ranges
                .iter()
                .any(|&(start, end)| start <= span.start && span.end <= end)
    }

    pub fn missing_within(&self, span: &Range<u64>) -> Vec<Range<u64>> {
        let mut missing = Vec::new();
        let mut cursor = span.start;
        for &(start, end) in &self.ranges {
            if start > cursor && cursor < span.end {
                missing.push(cursor..span.end.min(start));
            }
            cursor = cursor.max(end);
        }
        if cursor < span.end {
            missing.push(cursor..span.end);
        }
        missing
    }

    pub fn is_complete(&self) -> bool {
        match self.total_len {
            Some(0) => self.ranges.is_empty(),
            Some(len) => self.ranges == [(0, len)],
            None => false,
        }
    }

    pub fn to_json(&self) -> String {
        let total = self
            .total_len
            .map_or_else(|| "null".to_owned(), |len| len.to_string());
        let ranges = self
            .ranges
            .iter()
            .map(|(start, end)| format!("[{start},{end}]"))
            .collect::<Vec<_>>()
            .join(",");
        format!("{{\"total_len\":{total},\"ranges\":[{ranges}]}}")
    }

    pub fn from_json(text: &str) -> Option<Self> {
        let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        let total_len = parse_total(field(&compact, "\"total_len\":")?)?;
        let pairs = parse_pairs(field(&compact, "\"ranges\":")?)?;
        let mut manifest = Self::default();
        for (start, end) in pairs {
            manifest.insert(start..end).ok()?;
        }
        if let Some(len) = total_len {
            manifest.set_total_len(len).ok()?;
        }
        Some(manifest)
    }
}

fn coalesce(sorted: &[(u64, u64)]) -> Vec<(u64, u64)> {
    let mut merged: Vec<(u64, u64)> = Vec::new();
    for &(start, end) in sorted {
        match merged.last_mut() {
            Some(last) if start <= last.1 => last.1 = last.1.max(end),
            _ => merged.push((start, end)),
        }
    }
    merged
}

fn field<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let index = text.find(key)?;
    Some(&text[index + key.len()..])
}

fn parse_total(text: &str) -> Option<Option<u64>> {
    if text.starts_with("null") {
        return Some(None);
    }
    Some(Some(leading_number(text)?.0))
}

fn leading_number(text: &str) -> Option<(u64, &str)> {
    let digits = text.len() - text.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    let value = text.get(..digits)?.parse().ok()?;
    Some((value, &text[digits..]))
}

fn parse_pair(text: &str) -> Option<((u64, u64), &str)> {
    let (start, rest) = leading_number(text.strip_prefix('[')?)?;
    let (end, rest) = leading_number(rest.strip_prefix(',')?)?;
    Some(((start, end), rest.strip_prefix(']')?))
}

fn parse_pairs(text: &str) -> Option<Vec<(u64, u64)>> {
    let mut rest = text.strip_prefix('[')?;
    let mut pairs = Vec::new();
    while rest.starts_with('[') {
        let (pair, remaining) = parse_pair(rest)?;
        pairs.push(pair);
        rest = remaining.trim_start_matches(',');
    }
    rest.strip_prefix(']')?;
    Some(pairs)
}
