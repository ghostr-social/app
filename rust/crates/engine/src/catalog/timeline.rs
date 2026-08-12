use super::Catalog;
use crate::media_timeline::MediaTimeline;
use crate::representation::RepresentationBinding;

impl Catalog {
    /// Installs timing only for the representation that produced the bytes.
    pub fn learn_timeline_for(
        &mut self,
        binding: &RepresentationBinding,
        timeline: MediaTimeline,
    ) -> bool {
        let Some(entry) = self.current_entry(binding) else {
            return false;
        };
        entry.timeline = Some(timeline);
        entry.tail_timeline_needed = false;
        true
    }

    /// Records that a bounded head inspection found no timing metadata.
    pub fn require_tail_timeline_for(&mut self, binding: &RepresentationBinding) -> bool {
        let Some(entry) = self.current_entry(binding) else {
            return false;
        };
        entry.tail_timeline_needed = true;
        true
    }

    fn current_entry(
        &mut self,
        binding: &RepresentationBinding,
    ) -> Option<&mut super::CatalogEntry> {
        let entry = self.entries.get_mut(binding.post())?;
        (&entry.binding == binding).then_some(entry)
    }
}
