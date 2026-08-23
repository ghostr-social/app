use anyhow::{ensure, Result};

pub(super) fn discovered_total(length: Option<u64>, maximum: u64) -> Result<Option<u64>> {
    ensure!(maximum > 0, "whole response cap must be positive");
    let Some(length) = length else {
        return Ok(None);
    };
    ensure!(length > 0, "whole response length must be positive");
    Ok((length > maximum).then_some(length))
}
