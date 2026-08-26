use super::*;

impl Catalog {
    pub fn select_rendition(
        &mut self,
        post: &PostId,
        network: NetworkConditions,
        observation: PlaybackObservation,
        target: BufferTarget,
    ) -> Option<RepresentationBinding> {
        self.select_rendition_excluding(
            post,
            RenditionSelection::new(network, observation, target),
            &HashSet::new(),
        )
    }
}
