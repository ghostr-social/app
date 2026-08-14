import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_media_identity.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

/// Exclusive ownership of one older-page cursor.
final class FeedPageLease {
  const FeedPageLease(this.cursor);

  final DateTime cursor;
}

/// Tracks how far into the past the feed has been paged and keeps a single
/// older-page request in flight at a time.
final class FeedPagination {
  DateTime? _cursor;
  FeedPageLease? _active;

  /// Rebases the cursor just below the oldest post of a fresh load.
  void restartFrom(List<VideoPost> posts) {
    DateTime? oldest;
    for (final post in posts) {
      if (oldest == null || post.feedActivityAt.isBefore(oldest)) {
        oldest = post.feedActivityAt;
      }
    }
    _cursor = oldest?.subtract(const Duration(seconds: 1));
    _active = null;
  }

  /// Claims the cursor for one request; null when exhausted or already busy.
  FeedPageLease? beginLoad() {
    if (_active != null) return null;
    final cursor = _cursor;
    if (cursor == null) return null;
    return _active = FeedPageLease(cursor);
  }

  void completeLoad(FeedPageLease lease, VideoFeedPage page) {
    if (!identical(_active, lease)) return;
    _cursor = page.nextOlderThan;
    _active = null;
  }

  void failLoad(FeedPageLease lease) {
    if (identical(_active, lease)) _active = null;
  }

  /// Appends the page posts whose videos the list does not already play.
  static List<VideoPost> appendNew(
    List<VideoPost> current,
    List<VideoPost> incoming,
  ) {
    final seen = SeenVideoIdentities(current);
    return [
      ...current,
      for (final post in incoming)
        if (seen.add(post)) post,
    ];
  }
}
