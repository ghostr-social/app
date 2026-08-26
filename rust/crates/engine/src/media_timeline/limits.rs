use core::mem::size_of;

use super::{MediaSegment, TimelineError, TimelineParseControl};

const MAXIMUM_INPUT_BYTES: usize = 8 * 1024 * 1024;
const MAXIMUM_SEGMENTS: usize = 4_096;
const MAXIMUM_STRUCTURAL_BOX_BYTES: usize = 4 * 1024 * 1024;
const MAXIMUM_BOXES: usize = 4_096;
const MAXIMUM_DEPTH: usize = 8;
const MAXIMUM_TRACKS: usize = 16;
const MAXIMUM_SAMPLES: usize = 200_000;
const MAXIMUM_TABLE_WORK: usize = 800_000;
const MAXIMUM_ALLOCATION_BYTES: usize = 8 * 1024 * 1024;
const SCAN_CANCELLATION_BLOCK: usize = 4 * 1024;
const WORK_CANCELLATION_BLOCK: usize = 512;

pub(super) struct ParserBudget<'a> {
    boxes: usize,
    tracks: usize,
    samples: usize,
    table_work: usize,
    allocation_bytes: usize,
    scan_since_poll: usize,
    work_since_poll: usize,
    control: &'a dyn TimelineParseControl,
}

impl<'a> ParserBudget<'a> {
    pub(super) fn new(
        segments: &[MediaSegment<'_>],
        control: &'a dyn TimelineParseControl,
    ) -> Result<Self, TimelineError> {
        if segments.len() > MAXIMUM_SEGMENTS {
            return Err(TimelineError::ResourceLimit);
        }
        let bytes = segments.iter().try_fold(0_usize, |total, segment| {
            total.checked_add(segment.bytes.len())
        });
        if bytes.is_none_or(|bytes| bytes > MAXIMUM_INPUT_BYTES) {
            return Err(TimelineError::ResourceLimit);
        }
        let parser = Self {
            boxes: 0,
            tracks: 0,
            samples: 0,
            table_work: 0,
            allocation_bytes: 0,
            scan_since_poll: 0,
            work_since_poll: 0,
            control,
        };
        parser.checkpoint()?;
        Ok(parser)
    }

    pub(super) fn box_at(&mut self, bytes: usize, depth: usize) -> Result<(), TimelineError> {
        self.box_header(depth)?;
        if bytes > MAXIMUM_STRUCTURAL_BOX_BYTES {
            return Err(TimelineError::ResourceLimit);
        }
        Ok(())
    }

    pub(super) fn box_header(&mut self, depth: usize) -> Result<(), TimelineError> {
        self.checkpoint()?;
        if depth > MAXIMUM_DEPTH {
            return Err(TimelineError::ResourceLimit);
        }
        Self::take(&mut self.boxes, 1, MAXIMUM_BOXES)
    }

    pub(super) fn track(&mut self) -> Result<(), TimelineError> {
        self.checkpoint()?;
        Self::take(&mut self.tracks, 1, MAXIMUM_TRACKS)
    }

    pub(super) fn samples(&mut self, count: usize) -> Result<(), TimelineError> {
        self.checkpoint()?;
        Self::take(&mut self.samples, count, MAXIMUM_SAMPLES)
    }

    pub(super) fn table_work(&mut self, count: usize) -> Result<(), TimelineError> {
        self.checkpoint()?;
        Self::take(&mut self.table_work, count, MAXIMUM_TABLE_WORK)
    }

    pub(super) fn scan(&mut self, bytes: usize) -> Result<(), TimelineError> {
        let mut remaining = bytes;
        while self.scan_since_poll.saturating_add(remaining) >= SCAN_CANCELLATION_BLOCK {
            let until_poll = SCAN_CANCELLATION_BLOCK - self.scan_since_poll;
            remaining = remaining.saturating_sub(until_poll);
            self.scan_since_poll = 0;
            self.checkpoint()?;
        }
        self.scan_since_poll = self.scan_since_poll.saturating_add(remaining);
        Ok(())
    }

    pub(super) fn work(&mut self, items: usize) -> Result<(), TimelineError> {
        self.work_since_poll = self.work_since_poll.saturating_add(items);
        if self.work_since_poll >= WORK_CANCELLATION_BLOCK {
            self.work_since_poll = 0;
            self.checkpoint()?;
        }
        Ok(())
    }

    pub(super) fn reserve<T>(
        &mut self,
        target: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), TimelineError> {
        self.checkpoint()?;
        let bytes = size_of::<T>()
            .checked_mul(additional)
            .ok_or(TimelineError::ResourceLimit)?;
        Self::take(&mut self.allocation_bytes, bytes, MAXIMUM_ALLOCATION_BYTES)?;
        target
            .try_reserve_exact(additional)
            .map_err(|_allocation_error| TimelineError::ResourceLimit)
    }

    pub(super) fn vector<T>(&mut self, count: usize) -> Result<Vec<T>, TimelineError> {
        let mut values = Vec::new();
        self.reserve(&mut values, count)?;
        Ok(values)
    }

    pub(super) fn push<T>(&mut self, target: &mut Vec<T>, value: T) -> Result<(), TimelineError> {
        self.reserve(target, 1)?;
        target.push(value);
        Ok(())
    }

    pub(super) fn resize<T: Clone>(
        &mut self,
        target: &mut Vec<T>,
        length: usize,
        value: T,
    ) -> Result<(), TimelineError> {
        while target.len() < length {
            let next = target
                .len()
                .saturating_add(WORK_CANCELLATION_BLOCK)
                .min(length);
            target.resize(next, value.clone());
            self.work(WORK_CANCELLATION_BLOCK)?;
        }
        Ok(())
    }

    fn checkpoint(&self) -> Result<(), TimelineError> {
        if self.control.is_cancelled() {
            Err(TimelineError::Cancelled)
        } else {
            Ok(())
        }
    }

    fn take(total: &mut usize, amount: usize, maximum: usize) -> Result<(), TimelineError> {
        let Some(next) = total.checked_add(amount).filter(|next| *next <= maximum) else {
            return Err(TimelineError::ResourceLimit);
        };
        *total = next;
        Ok(())
    }
}
