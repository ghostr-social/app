/// Decides which feed answer still matters.
///
/// Loads, reloads, refreshes and hunts overlap: a slow relay can answer a
/// question the viewer has already moved on from. Every attempt takes a
/// number, and only the newest one may still change what is on screen.
final class FeedLoad<T extends Object> {
  const FeedLoad(this.request, this.value);

  final int request;
  final T value;
}

final class FeedLoads {
  int _newest = 0;

  /// Runs [attempt] as the newest one. The answer comes back null when a
  /// later attempt took over while this one was travelling.
  Future<T?> newest<T extends Object>(Future<T> Function() attempt) async {
    return (await leased(attempt))?.value;
  }

  Future<FeedLoad<T>?> leased<T extends Object>(
    Future<T> Function() attempt,
  ) async {
    final request = take();
    final answer = await attempt();
    return accepts(request) ? FeedLoad(request, answer) : null;
  }

  /// Claims the newest attempt, superseding every earlier one.
  int take() => ++_newest;

  /// The attempt currently in charge, without claiming a new one.
  int get pending => _newest;

  /// Whether [request] is still the newest attempt.
  bool accepts(int request) => request == _newest;
}
