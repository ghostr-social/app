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
            .expect("test fixture precondition must hold");
        self.context
            .store
            .set_total_len("clip", 16)
            .await
            .expect("test fixture precondition must hold");
        self.context
            .store
            .write_range("clip", 0, &[7; 16])
            .await
            .expect("test fixture precondition must hold");
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
        self.context
            .store
            .finalize("clip", None)
            .await
            .expect("test fixture precondition must hold");
        let input = self
            .context
            .store
            .media_snapshot("clip")
            .await
            .expect("test fixture precondition must hold");
        let publication = TransformPublication::try_new(
            TransformFence::new(
                input
                    .binding()
                    .expect("test fixture precondition must hold")
                    .clone(),
                input.revision(),
            ),
            TransformKind::Remux,
            vec![8; 16],
            16,
        )
        .expect("test fixture precondition must hold");
        assert!(self
            .context
            .store
            .publish_transform(publication)
            .await
            .expect("test fixture precondition must hold"));
        let derived = self
            .context
            .store
            .media_snapshot("clip")
            .await
            .expect("test fixture precondition must hold");
        self.representation = derived
            .binding()
            .expect("test fixture precondition must hold")
            .representation()
            .fingerprint()
            .to_owned();
        self.asset = self
            .context
            .capabilities
            .issue(&derived)
            .await
            .expect("test fixture precondition must hold")
            .as_str()
            .to_owned();
    }

    pub(super) fn preparation_context(&self) -> PreparationContext {
        PreparationContext {
            endpoint: "127.0.0.1:8080".to_owned(),
            store: std::sync::Arc::clone(&self.context.store),
            capabilities: self.context.capabilities.clone(),
            delivery: self.context.delivery.clone(),
            tracked: self.context.tracked.clone(),
            cache: self.context.cache.clone(),
        }
    }

    async fn refresh_authority(&mut self) {
        let snapshot = self
            .context
            .store
            .media_snapshot("clip")
            .await
            .expect("test fixture precondition must hold");
        self.representation = snapshot
            .binding()
            .expect("test fixture precondition must hold")
            .representation()
            .fingerprint()
            .to_owned();
        self.asset = self
            .context
            .capabilities
            .issue(&snapshot)
            .await
            .expect("test fixture precondition must hold")
            .as_str()
            .to_owned();
    }
}
