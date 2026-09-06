import 'dart:async';

import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import 'fakes.dart';

final class GatedFeedHistory extends FakeWatchHistoryRepository {
  final pending = Completer<void>();
  final release = Completer<void>();
  var writes = 0;

  @override
  Future<void> record(WatchHistoryEntry entry) async {
    writes++;
    if (writes == 2) {
      pending.complete();
      await release.future;
    }
    await super.record(entry);
  }

  void unblock() {
    if (!release.isCompleted) release.complete();
  }
}
