use super::DeliveryWorker;
use crate::manager::transfers::{InternalEvent, MaintenanceEvent, TransferEvent};

impl DeliveryWorker {
    pub(super) async fn apply_internal(&mut self, event: InternalEvent) {
        match event {
            InternalEvent::ImmediateReplan => self.consume_immediate_replan(),
            InternalEvent::NetworkRefill(wake) => {
                self.network_refill_timer.finish(wake);
            }
            InternalEvent::Transfer(transfer) => self.apply_transfer(transfer).await,
            InternalEvent::Segmented(done) => self.finish_segmented(*done),
            InternalEvent::Transform(done) => self.finish_transform_job(done),
            InternalEvent::HedgeTail(wake) => self.consume_hedge_tail_wake(wake),
            InternalEvent::Maintenance(maintenance) => self.apply_maintenance(maintenance).await,
            InternalEvent::TrafficChanged => self.absorb_traffic(),
        }
        self.refresh_observation_posts();
    }

    pub(super) fn refresh_observation_posts(&mut self) {
        let mut posts = self.downloads.body_posts();
        posts.extend(
            self.probes
                .active_identities()
                .into_iter()
                .map(|identity| identity.post().clone()),
        );
        self.state.set_observation_posts(posts);
    }

    async fn apply_transfer(&mut self, event: TransferEvent) {
        match event {
            TransferEvent::ChunkDone(done) => self.finish_chunk(done).await,
            TransferEvent::ProbeDone(done) => self.finish_probe(done).await,
            TransferEvent::ResponseObserved(observed) => self.observe_response(*observed).await,
        }
    }

    async fn apply_maintenance(&mut self, event: MaintenanceEvent) {
        match event {
            MaintenanceEvent::CooldownOver(post, cooldown) => self.finish_cooldown(post, cooldown),
            MaintenanceEvent::SaveStats => {
                self.keeper.save_now().await;
                let evidence = self.state.catalog().evidence_state();
                self.reliability.save_now(&evidence).await;
                self.save_capability().await;
            }
            MaintenanceEvent::SaveQoe => self.qoe.save_now().await,
            MaintenanceEvent::StoreCapacityChanged(value) => self.resume_store_capacity(value),
        }
    }
}
