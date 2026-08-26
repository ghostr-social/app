use super::*;

impl RelayRoleIo {
    pub(crate) fn with_registration(
        client: Arc<Client>,
        registration: Arc<dyn RelayRegistration>,
    ) -> Self {
        let removal = Arc::new(SdkRelayRemoval {
            client: std::sync::Arc::clone(&client),
        });
        Self {
            client,
            removal,
            registration,
        }
    }
}
