use anyhow::{bail, ensure, Result};
use std::collections::HashSet;

pub fn quoted_attribute(line: &str, name: &str) -> Result<Option<(usize, usize)>> {
    let Some((_, attributes)) = line.split_once(':') else {
        return Ok(None);
    };
    let offset = line.len() - attributes.len();
    let value = find_attribute(attributes, name)?;
    Ok(value.map(|(start, end)| (offset + start, offset + end)))
}

fn find_attribute(attributes: &str, name: &str) -> Result<Option<(usize, usize)>> {
    let mut scan = AttributeScan::new(name);
    let mut start = 0;
    let mut quoted = false;
    for (index, value) in attributes.char_indices() {
        if value == '"' {
            quoted = !quoted;
        } else if value == ',' && !quoted {
            scan.visit(attributes, start, index)?;
            start = index + 1;
        }
    }
    if quoted {
        bail!("HLS attribute list has an unterminated quote");
    }
    scan.visit(attributes, start, attributes.len())?;
    Ok(scan.found)
}

struct AttributeScan {
    target: String,
    found: Option<(usize, usize)>,
    names: HashSet<String>,
}

impl AttributeScan {
    fn new(target: &str) -> Self {
        Self {
            target: target.to_owned(),
            found: None,
            names: HashSet::new(),
        }
    }

    fn visit(&mut self, attributes: &str, start: usize, end: usize) -> Result<()> {
        let field = &attributes[start..end];
        let trimmed = field.trim_start();
        let (name, value) = attribute_parts(trimmed)?;
        ensure!(
            self.names.insert(name.to_owned()),
            "duplicate HLS attribute {name}"
        );
        if name != self.target {
            return Ok(());
        }
        let value = quoted_value(value, name)?;
        let value_start = start + field.len() - trimmed.len() + name.len() + 2;
        self.found = Some((value_start, value_start + value.len()));
        Ok(())
    }
}

fn attribute_parts(field: &str) -> Result<(&str, &str)> {
    let (name, value) = field
        .split_once('=')
        .ok_or_else(|| anyhow::anyhow!("malformed HLS attribute"))?;
    ensure!(
        !name.is_empty()
            && name
                .bytes()
                .all(|value| value.is_ascii_uppercase() || value.is_ascii_digit() || value == b'-'),
        "invalid HLS attribute name"
    );
    Ok((name, value))
}

fn quoted_value<'a>(value: &'a str, name: &str) -> Result<&'a str> {
    if !value.starts_with('"') || !value.ends_with('"') || value.len() < 2 {
        bail!("HLS {name} attribute must be a quoted string");
    }
    Ok(&value[1..value.len() - 1])
}
