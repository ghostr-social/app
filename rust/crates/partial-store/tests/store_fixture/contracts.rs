use ghostr_engine::adaptive::WholeBodyContract;

pub fn exact_response(expected_bytes: u64) -> WholeBodyContract {
    WholeBodyContract::Exact { expected_bytes }
}
