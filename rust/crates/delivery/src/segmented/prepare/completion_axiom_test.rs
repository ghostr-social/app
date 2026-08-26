use super::*;

impl PreparedComplete {
    pub(in crate::segmented) fn new(object: PreparedObject) -> Self {
        let mut hasher = CachedHlsGenerationHasher::new(
            &object.final_url,
            object.body.len() as u64,
            &object.cache,
        );
        hasher.update(&object.body);
        Self {
            object,
            generation: hasher.finish(),
        }
    }
}
