/// How aggressively Ghostr may fetch content over the network.
///
/// The level bounds the shared retrieval worker pool, giving one central
/// knob for data usage across feeds, search, and discovery.
enum DataUsageLevel {
  conservative('Conservative', 2),
  balanced('Balanced', 4),
  aggressive('Aggressive', 6);

  const DataUsageLevel(this.label, this.maxConcurrentRequests);

  final String label;
  final int maxConcurrentRequests;
}
