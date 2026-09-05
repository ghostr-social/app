use super::super::super::boxes::Atom;
use super::super::super::limits::ParserBudget;
use super::super::super::TimelineError;
use super::read::{byte, count_at, require_table_extent, u32_at};

pub(in crate::media_timeline::classic) struct SampleDependencies {
    offsets: Vec<i64>,
    sync: Option<Vec<u32>>,
}

impl SampleDependencies {
    pub(super) fn parse(
        atoms: &[Atom<'_>],
        samples: usize,
        budget: &mut ParserBudget<'_>,
    ) -> Result<Self, TimelineError> {
        let offsets = match super::find(atoms, b"ctts") {
            Some(atom) => composition(&atom, samples, budget)?,
            None => Vec::new(),
        };
        let sync = super::find(atoms, b"stss")
            .map(|atom| sync_samples(&atom, samples, budget))
            .transpose()?;
        Ok(Self { offsets, sync })
    }

    pub(in crate::media_timeline::classic) fn offset(&self, sample: usize) -> i64 {
        self.offsets.get(sample).copied().unwrap_or(0)
    }

    pub(in crate::media_timeline::classic) fn sync_before(&self, sample: usize) -> Option<u32> {
        let sample = sample as u32;
        let Some(sync) = &self.sync else {
            return Some(sample);
        };
        let count = sync.partition_point(|entry| *entry <= sample);
        count.checked_sub(1).map(|index| sync[index])
    }
}

fn composition(
    atom: &Atom<'_>,
    samples: usize,
    budget: &mut ParserBudget<'_>,
) -> Result<Vec<i64>, TimelineError> {
    let data = atom.payload();
    let version = byte(data, 0)?;
    if version > 1 {
        return Err(TimelineError::Unsupported);
    }
    let count = count_at(data, 4)?;
    budget.table_work(count)?;
    require_table_extent(data, count, 8)?;
    let mut offsets = budget.vector(samples)?;
    for index in 0..count {
        budget.work(1)?;
        let run = u32_at(data, 8 + index * 8)? as usize;
        let raw = u32_at(data, 12 + index * 8)?;
        let offset = if version == 1 {
            i64::from(raw as i32)
        } else {
            i64::from(raw)
        };
        let next = offsets
            .len()
            .checked_add(run)
            .ok_or(TimelineError::Malformed)?;
        if run == 0 || next > samples {
            return Err(TimelineError::Malformed);
        }
        budget.resize(&mut offsets, next, offset)?;
    }
    if offsets.len() != samples {
        return Err(TimelineError::Malformed);
    }
    Ok(offsets)
}

fn sync_samples(
    atom: &Atom<'_>,
    samples: usize,
    budget: &mut ParserBudget<'_>,
) -> Result<Vec<u32>, TimelineError> {
    let data = atom.payload();
    if byte(data, 0)? != 0 {
        return Err(TimelineError::Unsupported);
    }
    let count = count_at(data, 4)?;
    budget.table_work(count)?;
    require_table_extent(data, count, 4)?;
    if count > samples {
        return Err(TimelineError::Malformed);
    }
    let mut sync = budget.vector(count)?;
    for index in 0..count {
        budget.work(1)?;
        let number = u32_at(data, 8 + index * 4)?;
        let sample = number.checked_sub(1).ok_or(TimelineError::Malformed)?;
        if number as usize > samples || sync.last().is_some_and(|previous| *previous >= sample) {
            return Err(TimelineError::Malformed);
        }
        sync.push(sample);
    }
    Ok(sync)
}
