use super::*;

impl RelayPoolOwner {
    pub fn with_io(
        client: Arc<Client>,
        configuration: RelayPoolConfiguration,
        io: Arc<dyn RelayIo>,
    ) -> Self {
        Self::with_components(
            configuration,
            io,
            RelayRoleIo::sdk(client),
            Arc::new(RelayHealth::new()),
        )
    }
}
