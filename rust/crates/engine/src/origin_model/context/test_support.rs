use super::*;
use crate::origin_model::OriginEnvironment;

impl OriginQuery {
    pub(crate) fn with_environment(mut self, environment: OriginEnvironment) -> Self {
        self.environment = environment;
        self
    }
}
