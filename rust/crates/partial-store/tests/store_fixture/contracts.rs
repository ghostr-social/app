use ghostr_engine::adaptive::WholeBodyContract;

pub(in crate::tests) fn exact_response(expected_bytes: u64) -> WholeBodyContract {
    WholeBodyContract::Exact { expected_bytes }
}
