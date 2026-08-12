use crate::manager::workers::DownloadWorkers;

#[test]
fn clearing_workers_resets_the_admitted_capacity_snapshot() {
    let mut workers = DownloadWorkers::new();
    workers.reconcile(&[], 3);

    assert_eq!(workers.admitted_capacity(), 3);
    workers.clear();

    assert_eq!(workers.admitted_capacity(), 1);
}
