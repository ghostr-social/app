use super::player_preparation_authority_fixture::AuthorityFixture;
use crate::api::playback_preparation_stream::PreparationContext;
use ghostr_engine::adaptive::TransformKind;
use ghostr_partial_store::partial_range_store::{TransformFence, TransformPublication};

impl AuthorityFixture {
    pub(super) async fn publish_derived_representation(&mut self) {
        self.context.store.finalize("clip", None).await.unwrap();
        let input = self.context.store.media_snapshot("clip").await.unwrap();
        let publication = TransformPublication::try_new(
            TransformFence::new(input.binding().unwrap().clone(), input.revision()),
            TransformKind::Remux,
            vec![8; 16],
            16,
        )
        .unwrap();
        assert!(self
            .context
            .store
            .publish_transform(publication)
            .await
            .unwrap());
        let derived = self.context.store.media_snapshot("clip").await.unwrap();
        self.representation = derived
            .binding()
            .unwrap()
            .representation()
            .fingerprint()
            .to_owned();
        self.asset = self
            .context
            .capabilities
            .issue(&derived)
            .await
            .unwrap()
            .as_str()
            .to_owned();
    }

    pub(super) fn preparation_context(&self) -> PreparationContext {
        PreparationContext {
            endpoint: "127.0.0.1:8080".to_owned(),
            store: self.context.store.clone(),
            capabilities: self.context.capabilities.clone(),
            delivery: self.context.delivery.clone(),
            tracked: self.context.tracked.clone(),
            cache: self.context.cache.clone(),
        }
    }
}
