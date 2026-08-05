import 'dart:async';
import 'dart:developer';

import 'package:ghostr/features/video_catalog/data/rust_feed_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_page_reader.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_port.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_update_queue.dart';

/// One Rust feed held open across pulls (plan §5.3). The engine keeps
/// filing pages into an open feed — the scheduler prefetches older
/// pages and widens the active query — so its newest snapshot is
/// everything discovery gathered this session.
/// A returning pull reads that snapshot; only a feed that never
/// settled a page waits for relays.
final class RustFeedSession {
  factory RustFeedSession({
    required RustFeedPort port,
    required RustFeedId feedId,
    required RustFeedSpecKey specKey,
    required Duration deadline,
  }) {
    final updates = RustFeedUpdateQueue(port.feedUpdates(feedId));
    return RustFeedSession._(
      RustFeedSessionLink(port: port, feedId: feedId, specKey: specKey),
      updates,
      RustFeedPageReader(updates, deadline: deadline),
    );
  }

  RustFeedSession._(this._link, this._updates, this._reader);

  final RustFeedSessionLink _link;
  final RustFeedUpdateQueue _updates;
  final RustFeedPageReader _reader;
  RustFeedPage? _page;

  /// The spec this feed was opened for; two pulls that name it share
  /// this session.
  RustFeedSpecKey get specKey => _link.specKey;

  /// Whether the snapshot stream is still alive. A finished one means
  /// Rust ended the feed, or its watcher died: reopen instead of
  /// serving from it.
  bool get isLive => !_updates.isFinished;

  /// The newest page this feed settled on, or null while it has never
  /// settled one. Reading it takes in whatever landed since the last
  /// pull, so a returning caller sees the engine's latest rows.
  RustFeedPage? get warmPage {
    final page = _page;
    if (page == null) return null;
    final pending = _updates.drain();
    if (pending != null && pending.revision >= page.revision) {
      _page = (revision: pending.revision, posts: pending.posts);
    }
    return _page;
  }

  /// Waits for the feed's first settled page.
  Future<RustFeedPage> firstPage() async => _settled(await _reader.firstPage());

  /// Claims the page below [cursor] and waits for it; a feed that
  /// reports itself exhausted keeps [loaded].
  Future<RustFeedPage> olderPage(RustFeedPage loaded, BigInt cursor) async {
    final more = await _link.port.loadMore(
      _link.feedId,
      olderThanSecs: cursor,
    );
    if (!more) return loaded;
    return _settled(await _reader.olderPage(loaded));
  }

  /// Asks the engine for another page without waiting for it: the rows
  /// land in this feed's snapshot and answer the pull after this one.
  /// The engine picks the cursor — it tracks how deep the feed got.
  void deepen() => unawaited(_deepened());

  Future<void> close() {
    // Fire-and-forget: a feed whose stream never ends must not hold
    // the close behind its own cancellation.
    unawaited(_updates.dispose());
    return _link.port.closeFeed(_link.feedId);
  }

  /// Revision zero is the baseline snapshot Rust publishes before any
  /// retrieval lands, and it is what the reader hands back when no page
  /// settled in time (rust_feed_page_reader.dart): neither is warm
  /// state a later pull may answer with.
  RustFeedPage _settled(RustFeedPage page) {
    if (page.revision > BigInt.zero) _page = page;
    return page;
  }

  Future<void> _deepened() async {
    try {
      await _link.port.loadMore(_link.feedId);
    } on Object catch (error, stackTrace) {
      log(
        'The Rust feed could not be deepened.',
        name: 'ghostr.video.rustfeed',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}

/// What a session needs to reach its feed in the engine.
final class RustFeedSessionLink {
  const RustFeedSessionLink({
    required this.port,
    required this.feedId,
    required this.specKey,
  });

  final RustFeedPort port;
  final RustFeedId feedId;
  final RustFeedSpecKey specKey;
}
