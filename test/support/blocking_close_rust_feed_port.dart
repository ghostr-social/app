import 'dart:async';

import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_port.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

final class BlockingCloseRustFeedPort implements RustFeedPort {
  BlockingCloseRustFeedPort(this.update);

  final FfiFeedUpdate update;
  final List<String?> openedViewers = <String?>[];
  final Map<RustFeedId, StreamController<FfiFeedUpdate>> _feeds =
      <RustFeedId, StreamController<FfiFeedUpdate>>{};
  final Completer<void> _closeStarted = Completer<void>();
  final Completer<void> _closeRelease = Completer<void>();

  Future<void> get closeStarted => _closeStarted.future;

  @override
  Future<RustFeedAccountSession> captureSession(
    NostrPublicKeyHex? expectedAccount,
  ) async {
    return RustFeedAccountSession(
      account: expectedAccount,
      generation: RustNostrSessionGeneration.fromBridge(BigInt.zero),
    );
  }

  @override
  Future<RustFeedId> openFeed(
    FfiFeedSpec spec,
    RustFeedAccountSession session,
  ) async {
    openedViewers.add(session.account?.value);
    final feedId = RustFeedId.parse('${openedViewers.length}');
    final updates = StreamController<FfiFeedUpdate>()..add(update);
    _feeds[feedId] = updates;
    return feedId;
  }

  @override
  Stream<FfiFeedUpdate> feedUpdates(RustFeedId feedId) {
    return _feeds[feedId]!.stream;
  }

  @override
  Future<bool> loadMore(RustFeedId feedId, {BigInt? olderThanSecs}) async {
    return false;
  }

  @override
  Future<void> closeFeed(RustFeedId feedId) async {
    if (feedId.value == '1' && !_closeRelease.isCompleted) {
      if (!_closeStarted.isCompleted) _closeStarted.complete();
      await _closeRelease.future;
    }
    await _feeds.remove(feedId)?.close();
  }

  void releaseClose() => _closeRelease.complete();

  Future<void> dispose() async {
    if (!_closeRelease.isCompleted) _closeRelease.complete();
    for (final updates in _feeds.values) {
      await updates.close();
    }
    _feeds.clear();
  }
}
