Future<void> drainTestMicrotasks([int turns = 5]) async {
  for (var turn = 0; turn < turns; turn += 1) {
    await Future<void>.delayed(Duration.zero);
  }
}
