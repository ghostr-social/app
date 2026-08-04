import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_fetcher.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_loads.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_pagination.dart';

/// What one dig into the past brought up.
sealed class FeedDig {
  const FeedDig();
}

/// Older posts to append.
final class FeedDigPage extends FeedDig {
  const FeedDigPage(this.posts);

  final List<VideoPost> posts;
}

/// The past could not be read; the reason is for the viewer.
final class FeedDigFailed extends FeedDig {
  const FeedDigFailed(this.message);

  final String message;
}

/// Nothing to do: the past ran dry, a dig is already in flight, or a newer
/// feed took over while this one was travelling.
final class FeedDigSkipped extends FeedDig {
  const FeedDigSkipped();
}

/// Decides when the feed must dig into the past.
///
/// The viewer must always have a queue of unwatched videos ahead of them, so
/// whenever the buffer runs short the backfill goes one page older — one dig
/// at a time — until the buffer refills or the past runs dry.
final class FeedBackfill {
  FeedBackfill(this._fetch, this._loads, {this.bufferTarget = 10});

  final FeedFetcher _fetch;
  final FeedLoads _loads;
  final _pagination = FeedPagination();

  /// How many unwatched videos should stay queued ahead of the viewer.
  final int bufferTarget;

  /// Rebases on a freshly loaded feed.
  void restartFrom(List<VideoPost> posts) => _pagination.restartFrom(posts);

  /// Whether the queue ahead of the viewer has run short.
  bool isStarved(FeedRoster roster) => roster.ahead < bufferTarget;

  /// Digs one page further into the past.
  Future<FeedDig> dig(FeedKind kind) async {
    final cursor = _pagination.beginLoad();
    if (cursor == null) return const FeedDigSkipped();
    final request = _loads.pending;
    final result = await _fetch.older(kind, cursor);
    if (result is FeedUnavailable) {
      _pagination.failLoad();
      return FeedDigFailed(result.describe());
    }
    if (!_loads.accepts(request)) return const FeedDigSkipped();
    final page = (result as FeedFetched).page;
    _pagination.completeLoad(page);
    return FeedDigPage(page.posts);
  }
}
