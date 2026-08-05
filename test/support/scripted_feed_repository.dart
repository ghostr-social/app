import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

/// Serves scripted head loads in order; `null` entries throw.
class ScriptedFeedRepository implements VideoFeedRepository {
  ScriptedFeedRepository({required List<List<VideoPost>?> loads})
    : _loads = List<List<VideoPost>?>.of(loads);

  final List<List<VideoPost>?> _loads;
  int loadCalls = 0;
  final loadExclusions = <bool>[];

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    loadCalls += 1;
    loadExclusions.add(excludeWatched);
    final page = _loads.isEmpty ? const <VideoPost>[] : _loads.removeAt(0);
    if (page == null) throw StateError('relay down');
    return page;
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async {
    return VideoFeedPage(posts: const <VideoPost>[]);
  }
}
