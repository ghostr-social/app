import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_repository.dart';
import 'package:ghostr/features/watch_history/domain/watched_video_index.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

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
  Future<List<VideoPost>> filterUnwatched(List<VideoPost> posts) async {
    final watched = WatchedVideoIndex(entries);
    return posts.where((post) => !watched.contains(post)).toList();
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
