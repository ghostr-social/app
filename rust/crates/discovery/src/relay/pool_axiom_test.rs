use super::*;

impl RelayPoolOwner {
    pub(crate) fn with_role_io(
        configuration: RelayPoolConfiguration,
        io: Arc<dyn RelayIo>,
        roles: RelayRoleIo,
    ) -> Self {
        Self::with_components(configuration, io, roles, Arc::new(RelayHealth::new()))
    }
    pub(crate) async fn read(
        &self,
        request: RelayReadRequest,
    ) -> Result<crate::relay::io::RelayReadResult, PlanFailure> {
        self.begin_route(request.session).await?.read(request).await
    }
    pub(crate) async fn broadcast(&self, request: RelayBroadcastRequest) -> anyhow::Result<()> {
        let route = self
            .begin_route(request.session)
            .await
            .map_err(|failure| anyhow::anyhow!(failure.message))?;
        route.broadcast(request).await
    }
}
