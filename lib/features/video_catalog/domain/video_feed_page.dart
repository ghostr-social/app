import 'package:ghostr/features/video_catalog/domain/video_post.dart';

/// One older slice of the feed plus the cursor for the slice after it.
class VideoFeedPage {
  VideoFeedPage({required List<VideoPost> posts, this.nextOlderThan})
      : posts = List<VideoPost>.unmodifiable(posts);

  final List<VideoPost> posts;

  /// Publication cutoff for the next page; null when the feed is exhausted.
  final DateTime? nextOlderThan;

  bool get hasMore => nextOlderThan != null;
}
