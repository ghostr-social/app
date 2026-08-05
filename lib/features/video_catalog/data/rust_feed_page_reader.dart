import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_update_queue.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

/// The slowest query a Rust discovery plan can contain, mirrored from
/// `DISCOVERY_QUERY_TIMEOUT` (rust/src/discovery/search_queries.rs).
/// Every search, hashtag and note-hunt query runs on it, and a page
/// becomes visible only once the whole plan resolves, so this is the
/// pipeline's own worst case for one page.
const rustDiscoveryQueryTimeout = Duration(seconds: 8);

/// The additive kind-0 lookup that follows a page's video queries,
/// mirrored from `FEED_QUERY_TIMEOUT`.
const rustProfileEnrichmentQueryTimeout = Duration(seconds: 5);

/// How long one request waits for a page to settle. A safety net for a
/// stuck pipeline, never the normal exit path: it sits above discovery
/// plus sequential profile enrichment with scheduler slack.
const rustFeedPageDeadline = Duration(seconds: 15);

/// Shared user-facing failure for Rust feed transport errors.
const rustFeedFailure = AppFailure('Could not load Nostr videos.');

/// One snapshot claimed as a page: its rows and the revision they came
/// from, so a later page can be recognised as strictly later.
typedef RustFeedPage = ({BigInt revision, List<FfiFeedPost> posts});

/// Reads pages out of one open feed's snapshot stream. Rust publishes a
/// full ordered list per revision and says how far the page got, so a
/// reader waits for the stage to leave `loading` instead of guessing
/// completeness from the row count — an empty settled page and a page
/// still in flight look identical otherwise.
final class RustFeedPageReader {
  const RustFeedPageReader(
    this._updates, {
    Duration deadline = rustFeedPageDeadline,
  }) : _deadline = deadline;

  final RustFeedUpdateQueue _updates;
  final Duration _deadline;

  /// The feed's first page. Empty when the stream ends or the deadline
  /// passes without a settled snapshot.
  Future<RustFeedPage> firstPage() async {
    const empty = <FfiFeedPost>[];
    return await _settled(_anyRevision) ??
        (revision: BigInt.zero, posts: empty);
  }

  /// The page after [loaded]: the next revision to settle. Snapshots are
  /// full lists, so an older page arrives as a later revision of the
  /// whole feed; [loaded] stands when nothing later settles in time.
  Future<RustFeedPage> olderPage(RustFeedPage loaded) async {
    final later = await _settled((update) => update.revision > loaded.revision);
    return later ?? loaded;
  }

  /// The first accepted snapshot that is done loading. Each wait gets
  /// the full deadline; the pipeline publishes one revision per
  /// completed retrieval, so this cannot spin.
  Future<RustFeedPage?> _settled(bool Function(FfiFeedUpdate) accept) async {
    while (true) {
      final update = await _updates.next(_deadline);
      if (update == null) return null;
      final page = _settledPage(update, accept);
      if (page != null) return page;
    }
  }

  static bool _anyRevision(FfiFeedUpdate update) => true;
}

RustFeedPage? _settledPage(
  FfiFeedUpdate update,
  bool Function(FfiFeedUpdate) accept,
) {
  if (!accept(update)) return null;
  if (update.stage == FfiFeedStage.failed) throw rustFeedFailure;
  if (update.stage == FfiFeedStage.loading) return null;
  return (revision: update.revision, posts: update.posts);
}
