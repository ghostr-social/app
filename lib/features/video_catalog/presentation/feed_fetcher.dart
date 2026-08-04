import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_failure_messages.dart';

/// What one feed request came back with.
sealed class FeedFetch {
  const FeedFetch();
}

/// A slice of the feed, with the cursor for the slice after it.
final class FeedFetched extends FeedFetch {
  const FeedFetched(this.page);

  final VideoFeedPage page;

  List<VideoPost> get posts => page.posts;
}

/// The relays gave nothing back.
final class FeedUnavailable extends FeedFetch {
  const FeedUnavailable(this.cause, this.stackTrace);

  final Object cause;
  final StackTrace stackTrace;

  /// The reason to show the viewer. Translating an unexpected cause also
  /// reports it, so silent retry paths must not ask for a reason they will
  /// never show.
  String describe() {
    final cause = this.cause;
    if (cause is AppFailure) return cause.message;
    return unexpectedFeedLoadMessage(cause, stackTrace);
  }
}

/// Reads the feed and turns every failure into something the caller can
/// show, so callers juggle outcomes instead of exceptions.
final class FeedFetcher {
  const FeedFetcher(this._feed);

  final VideoFeedRepository _feed;

  /// The newest posts the viewer has not watched yet.
  Future<FeedFetch> unwatched(FeedKind kind) {
    return _guarded(() => _feed.loadFeed(kind, excludeWatched: true));
  }

  /// The feed as the relays see it now, watched posts included, so the video
  /// the viewer is on can still be found.
  Future<FeedFetch> resync(FeedKind kind) {
    return _guarded(() => _feed.loadFeed(kind));
  }

  /// One page further into the past.
  Future<FeedFetch> older(FeedKind kind, DateTime olderThan) async {
    try {
      return FeedFetched(await _feed.loadOlderFeed(
        kind,
        olderThan: olderThan,
        excludeWatched: true,
      ));
    } on Object catch (error, stackTrace) {
      return FeedUnavailable(error, stackTrace);
    }
  }

  Future<FeedFetch> _guarded(Future<List<VideoPost>> Function() request) async {
    try {
      return FeedFetched(VideoFeedPage(posts: await request()));
    } on Object catch (error, stackTrace) {
      return FeedUnavailable(error, stackTrace);
    }
  }
}
