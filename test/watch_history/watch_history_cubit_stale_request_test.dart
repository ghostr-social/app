import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_repository.dart';
import 'package:ghostr/features/watch_history/presentation/watch_history_cubit.dart';

void main() {
  test('an older load finishing after clear keeps the cleared state', () async {
    final repository = _GatedWatchHistoryRepository();
    final cubit = WatchHistoryCubit(repository);
    addTearDown(cubit.close);

    final staleLoad = cubit.load();
    await cubit.clear();
    expect(cubit.state, isA<WatchHistoryEmpty>());

    repository.loadGate.complete();
    await staleLoad;

    expect(cubit.state, isA<WatchHistoryEmpty>());
  });
}

class _GatedWatchHistoryRepository implements WatchHistoryRepository {
  final loadGate = Completer<void>();

  @override
  WatchHistoryRepository snapshotForActiveAccount() => this;

  @override
  Future<List<WatchHistoryEntry>> load() async {
    await loadGate.future;
    return [
      WatchHistoryEntry(
        videoId: 'e:video-1',
        title: 'A relay-side banger',
        creatorName: 'Nora Relay',
        watchedAt: DateTime(2026, 3, 12, 10, 30),
      ),
    ];
  }

  @override
  Future<void> record(WatchHistoryEntry entry) async {}

  @override
  Future<void> clear() async {}
}
