import 'dart:async';
import 'package:ghostr/features/video_catalog/data/rust_feed_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_page_reader.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_port.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_update_queue.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

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

  /// The newest page this feed yielded, or null while it has never
  /// yielded one. Reading it takes in whatever landed since the last
  /// pull, so a returning caller sees the engine's latest rows.
  RustFeedPage? get warmPage {
    final page = _page;
    if (page == null) return null;
    final pending = _updates.drain();
    if (pending != null && pending.revision >= page.revision) {
      _remember((
        revision: pending.revision,
        posts: pending.posts,
        stage: pending.stage,
      ));
    }
    return _page;
  }

  /// Waits for the feed's first available page.
  Future<RustFeedPage> firstPage() async =>
      _remember(await _reader.firstPage());

  /// Claims an older page using Rust's raw-event cursor. A feed that reports
  /// itself exhausted keeps [loaded].
  Future<RustFeedPage> olderPage(RustFeedPage loaded) async {
    final more = await _link.port.loadMore(_link.feedId);
    if (!more) return loaded;
    return _remember(await _reader.olderPage(loaded));
  }

  /// Every useful full snapshot Rust publishes for this open session.
  Stream<RustFeedPage> watchPages() {
    return _updates
        .watch()
        .where(_isVisible)
        .map(_pageFromUpdate)
        .map(_remember);
  }

  Future<void> close() async {
    await _updates.dispose();
    await _link.port.closeFeed(_link.feedId);
  }

  /// Baselines and empty retry failures carry no page a later pull can use.
  /// The failure is still visible to passive watchers, while pull readers
  /// stay parked for the Rust-owned retry.
  RustFeedPage _remember(RustFeedPage page) {
    if (page.revision == BigInt.zero ||
        page.stage == FfiFeedStage.failed && page.posts.isEmpty) {
      return page;
    }
    final current = _page;
    if (current == null || page.revision >= current.revision) _page = page;
    return _page!;
  }

  bool _isVisible(FfiFeedUpdate update) {
    return update.stage != FfiFeedStage.loading || update.posts.isNotEmpty;
  }

  RustFeedPage _pageFromUpdate(FfiFeedUpdate update) {
    return (
      revision: update.revision,
      posts: update.posts,
      stage: update.stage,
    );
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
