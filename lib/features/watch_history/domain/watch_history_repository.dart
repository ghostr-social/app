import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

abstract interface class WatchHistoryRepository {
  WatchHistoryRepository snapshotForActiveAccount();

  Future<List<WatchHistoryEntry>> load();

  Future<void> record(WatchHistoryEntry entry);

  Future<void> clear();
}
