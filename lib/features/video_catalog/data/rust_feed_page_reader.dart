import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_update_queue.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

final _feedPageStopwatch = Stopwatch()..start();

Duration _feedPageElapsed() => _feedPageStopwatch.elapsed;

/// The slowest query a Rust discovery plan can contain, mirrored from
/// `DISCOVERY_QUERY_TIMEOUT` (rust/src/discovery/search_queries.rs).
/// Every search, hashtag and note-hunt query runs on it. Playable rows
/// can arrive before this timeout; it remains the empty-result horizon.
const rustDiscoveryQueryTimeout = Duration(seconds: 8);

/// The additive kind-0 lookup that follows a page's video queries,
/// mirrored from `FEED_QUERY_TIMEOUT`.
const rustProfileEnrichmentQueryTimeout = Duration(seconds: 5);

/// How long one request waits for rows or a settled page. A safety net
/// for a stuck pipeline, never the normal exit path: it sits above
/// discovery plus sequential profile enrichment with scheduler slack.
const rustFeedPageDeadline = Duration(seconds: 15);

/// Shared user-facing failure for Rust feed transport errors.
const rustFeedFailure = AppFailure('Could not load Nostr videos.');

/// One snapshot claimed as a page: its rows and the revision they came
/// from, so a later page can be recognised as strictly later.
typedef RustFeedPage = ({
  BigInt revision,
  List<FfiFeedPost> posts,
  FfiFeedStage stage,
});

/// Reads pages out of one open feed's snapshot stream. A non-empty loading
/// snapshot is useful provisional data; an empty one still waits until Rust
/// settles because it cannot distinguish "no matches" from "not yet".
final class RustFeedPageReader {
  factory RustFeedPageReader(
    RustFeedUpdateQueue updates, {
    Duration deadline = rustFeedPageDeadline,
    Duration Function()? elapsedClock,
  }) {
    return RustFeedPageReader._(updates, deadline, elapsedClock);
  }

  const RustFeedPageReader._(this._updates, this._deadline, this._elapsedClock);

  final RustFeedUpdateQueue _updates;
  final Duration _deadline;
  final Duration Function()? _elapsedClock;

  /// The feed's first available page. Empty only when the deadline passes.
  Future<RustFeedPage> firstPage() async {
    const empty = <FfiFeedPost>[];
    return await _available(_anyRevision) ??
        (revision: BigInt.zero, posts: empty, stage: FfiFeedStage.loading);
  }

  /// The page after [loaded]: the next revision to settle. Snapshots are
  /// full lists, so an older page arrives as a later revision of the
  /// whole feed; [loaded] stands when nothing later settles in time.
  Future<RustFeedPage> olderPage(RustFeedPage loaded) async {
    final later = await _available(
      (update) => update.revision > loaded.revision,
    );
    return later ?? loaded;
  }

  /// The first accepted snapshot that has rows or is done loading.
  Future<RustFeedPage?> _available(bool Function(FfiFeedUpdate) accept) async {
    final startedAt = _elapsed();
    while (true) {
      final remaining = _remaining(startedAt);
      if (remaining == Duration.zero) return null;
      final update = await _updates.next(remaining);
      if (update == null && _updates.isFinished) throw rustFeedFailure;
      if (update == null) return null;
      final page = _availablePage(update, accept);
      if (page != null) return page;
    }
  }

  Duration _remaining(Duration startedAt) {
    final remaining = _deadline - (_elapsed() - startedAt);
    return remaining > Duration.zero ? remaining : Duration.zero;
  }

  Duration _elapsed() => (_elapsedClock ?? _feedPageElapsed)();

  static bool _anyRevision(FfiFeedUpdate update) => true;
}

RustFeedPage? _availablePage(
  FfiFeedUpdate update,
  bool Function(FfiFeedUpdate) accept,
) {
  if (!accept(update)) return null;
  // Rust owns retry timing. A live stream may recover after this snapshot;
  // only a stream that ends without another page is a terminal failure.
  if (update.stage == FfiFeedStage.failed && update.posts.isEmpty) {
    return null;
  }
  if (update.stage == FfiFeedStage.loading && update.posts.isEmpty) return null;
  return (revision: update.revision, posts: update.posts, stage: update.stage);
}
