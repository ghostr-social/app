import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/watch_history/domain/video_watch_fingerprints.dart';

final class WatchHistoryCandidateFilter {
  const WatchHistoryCandidateFilter();

  List<VideoPost> apply({
    required List<VideoPost> posts,
    required List<VideoWatchFingerprints> candidates,
    required Set<String> stored,
    required DateTime? publishedThrough,
  }) {
    return List<VideoPost>.unmodifiable([
      for (var index = 0; index < posts.length; index += 1)
        if (_isFresh(posts[index], candidates[index], stored, publishedThrough))
          posts[index],
    ]);
  }

  bool _isFresh(
    VideoPost post,
    VideoWatchFingerprints candidate,
    Set<String> stored,
    DateTime? publishedThrough,
  ) {
    if (candidate.values.any(stored.contains)) return false;
    return publishedThrough == null ||
        post.publishedAt.isAfter(publishedThrough);
  }
}
