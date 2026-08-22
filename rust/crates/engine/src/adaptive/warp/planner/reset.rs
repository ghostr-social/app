use super::WarpPlanner;

impl WarpPlanner {
    /// Resets learned planning state without minting over live network reservations.
    pub fn reset_adaptation(&mut self) {
        let network = self.network.take();
        *self = Self::new(self.config);
        self.network = network;
    }
}
