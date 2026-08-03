import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_interaction_target.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

/// Tracks how far into the past the feed has been paged and keeps a single
/// older-page request in flight at a time.
final class FeedPagination {
  DateTime? _cursor;
  bool _inFlight = false;

  /// Rebases the cursor just below the oldest post of a fresh load.
  void restartFrom(List<VideoPost> posts) {
    DateTime? oldest;
    for (final post in posts) {
      if (oldest == null || post.publishedAt.isBefore(oldest)) {
        oldest = post.publishedAt;
      }
    }
    _cursor = oldest?.subtract(const Duration(seconds: 1));
    _inFlight = false;
  }

  /// Claims the cursor for one request; null when exhausted or already busy.
  DateTime? beginLoad() {
    if (_inFlight) return null;
    final cursor = _cursor;
    if (cursor == null) return null;
    _inFlight = true;
    return cursor;
  }

  void completeLoad(VideoFeedPage page) {
    _cursor = page.nextOlderThan;
    _inFlight = false;
  }

  void failLoad() {
    _inFlight = false;
  }

  /// Appends the page posts that are not already in the list.
  static List<VideoPost> appendNew(
    List<VideoPost> current,
    List<VideoPost> incoming,
  ) {
    final seen = <VideoInteractionTarget>{
      for (final post in current) VideoInteractionTarget.fromPost(post),
    };
    return [
      ...current,
      for (final post in incoming)
        if (seen.add(VideoInteractionTarget.fromPost(post))) post,
    ];
  }
}
