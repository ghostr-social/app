use super::player_preparation_authority_fixture::AuthorityFixture;
use super::selected_rendition_fixture::selected_rendition;
use crate::api::playback_preparation_stream::PreparationContext;
use ghostr_delivery::cache_registry::{CacheStatus, CacheVideo};
use ghostr_engine::adaptive::TransformKind;
use ghostr_partial_store::partial_range_store::{TransformFence, TransformPublication};

impl AuthorityFixture {
    pub(super) async fn publish_selected_representation(&mut self) -> String {
        let rendition = selected_rendition("clip");
        self.context
            .store
            .bind_representation(rendition.binding)
            .await
            .unwrap();
        self.context.store.set_total_len("clip", 16).await.unwrap();
        self.context
            .store
            .write_range("clip", 0, &[7; 16])
            .await
            .unwrap();
        self.context
            .tracked
            .insert("clip".to_owned(), rendition.advertised);
        self.context.cache.replace([CacheVideo {
            id: "clip".to_owned(),
            meta: rendition.selected,
            status: CacheStatus::Complete,
        }]);
        self.refresh_authority().await;
        rendition.advertised_representation
    }

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

    async fn refresh_authority(&mut self) {
        let snapshot = self.context.store.media_snapshot("clip").await.unwrap();
        self.representation = snapshot
            .binding()
            .unwrap()
            .representation()
            .fingerprint()
            .to_owned();
        self.asset = self
            .context
            .capabilities
            .issue(&snapshot)
            .await
            .unwrap()
            .as_str()
            .to_owned();
    }
}
