import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_repository.dart';

class FakeWatchHistoryRepository implements WatchHistoryRepository {
  FakeWatchHistoryRepository({List<WatchHistoryEntry>? entries})
      : entries = entries ?? <WatchHistoryEntry>[];

  final List<WatchHistoryEntry> entries;

  @override
  FakeWatchHistoryRepository snapshotForActiveAccount() => this;

  @override
  Future<List<WatchHistoryEntry>> load() async {
    return List<WatchHistoryEntry>.unmodifiable(entries);
  }

  @override
  Future<void> record(WatchHistoryEntry entry) async {
    entries.removeWhere((item) => item.videoId == entry.videoId);
    entries.insert(0, entry);
  }

  @override
  Future<void> clear() async {
    entries.clear();
  }
}
