import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

abstract interface class WatchHistoryRepository {
  WatchHistoryRepository snapshotForActiveAccount();

  Future<List<WatchHistoryEntry>> load();

  Future<List<VideoPost>> filterUnwatched(List<VideoPost> posts);

  Future<void> record(WatchHistoryEntry entry);

  Future<void> clear();
}
