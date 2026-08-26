use super::*;

impl NavigationPrediction {
    #[cfg(test)]
    pub(crate) fn backward_probability(self) -> f64 {
        self.backward
    }

    pub fn exit_probability(self) -> f64 {
        self.exit
    }
}

#[cfg(test)]
impl NavigationState {
    pub(crate) fn observations(&self) -> u64 {
        self.observations
    }
}
