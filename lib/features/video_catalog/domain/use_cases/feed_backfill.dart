import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_fetcher.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_loads.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_pagination.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

/// What one dig into the past brought up.
sealed class FeedDig {
  const FeedDig();
}

/// Older posts to append.
final class FeedDigPage extends FeedDig {
  const FeedDigPage(
    this.posts, {
    required this.hasMore,
    required this.cursorAdvanced,
  });

  final List<VideoPost> posts;
  final bool hasMore;
  final bool cursorAdvanced;
}

/// The past could not be read; presentation decides how to describe it.
final class FeedDigFailed extends FeedDig {
  const FeedDigFailed(this.failure);

  final FeedUnavailable failure;
}

/// Nothing to do: the past ran dry, a dig is already in flight, or a newer
/// feed took over while this one was travelling.
final class FeedDigSkipped extends FeedDig {
  const FeedDigSkipped({required this.retryable});

  final bool retryable;
}

/// Decides when the feed must dig into the past.
///
/// The viewer must always have a queue of unwatched videos ahead of them, so
/// whenever the buffer runs short the backfill goes one page older — one dig
/// at a time — until the buffer refills or the past runs dry.
final class FeedBackfill {
  FeedBackfill(
    this._fetch,
    this._loads, {
    this.bufferTarget = 10,
    this.dryPageLimit = 3,
  }) : assert(dryPageLimit > 0);

  final FeedFetcher _fetch;
  final FeedLoads _loads;
  final _pagination = FeedPagination();

  /// How many unwatched videos should stay queued ahead of the viewer.
  final int bufferTarget;

  /// Maximum cursor-advancing pages one drain may inspect without a new row.
  final int dryPageLimit;

  /// Rebases on a freshly loaded feed.
  void restartFrom(List<VideoPost> posts) => _pagination.restartFrom(posts);

  /// Whether the queue ahead of the viewer has run short.
  bool isStarved(FeedRoster roster) => roster.ahead < bufferTarget;

  /// Digs one page further into the past.
  Future<FeedDig> dig(FeedKind kind) async {
    final lease = _pagination.beginLoad();
    if (lease == null) {
      return FeedDigSkipped(retryable: !_pagination.isExhausted);
    }
    final request = _loads.pending;
    final result = await _fetch.older(kind, lease.cursor);
    if (result is FeedUnavailable) {
      _pagination.failLoad(lease);
      return FeedDigFailed(result);
    }
    if (!_loads.accepts(request)) {
      _pagination.failLoad(lease);
      return const FeedDigSkipped(retryable: true);
    }
    final page = (result as FeedFetched).page;
    final nextCursor = page.nextOlderThan;
    final cursorAdvanced = nextCursor?.isBefore(lease.cursor) ?? false;
    _pagination.completeLoad(lease, page);
    return FeedDigPage(
      page.posts,
      hasMore: page.hasMore,
      cursorAdvanced: cursorAdvanced,
    );
  }
}
