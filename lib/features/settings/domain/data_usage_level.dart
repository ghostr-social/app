/// How aggressively Ghostr may fetch content over the network.
///
/// The level bounds the shared retrieval worker pool and how many outbox
/// relays queries fan out to, giving one central knob for data usage
/// across feeds, search, and discovery.
enum DataUsageLevel {
  conservative('Conservative', 2, 6),
  balanced('Balanced', 4, 12),
  aggressive('Aggressive', 6, 18);

  const DataUsageLevel(
    this.label,
    this.maxConcurrentRequests,
    this.maxOutboxRelays,
  );

  final String label;
  final int maxConcurrentRequests;
  final int maxOutboxRelays;
}
