import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_port.dart';
import 'package:ghostr/src/rust/api/feed_control.dart' as feed_control;
import 'package:ghostr/src/rust/api/feed_types.dart';
import 'package:ghostr/src/rust/api/feed_updates_stream.dart' as feed_stream;

typedef RustFeedSessionCapture = Future<BigInt> Function({
  String? expectedAccountHex,
});
typedef RustFeedOpen = Future<String> Function({
  required FfiFeedSpec spec,
  String? expectedAccountHex,
  required BigInt expectedSessionGeneration,
});
typedef RustFeedWatch = Stream<FfiFeedUpdate> Function(
    {required String feedId});
typedef RustFeedMore = Future<bool> Function(
    {required String feedId, BigInt? olderThanSecs});
typedef RustFeedClose = Future<void> Function({required String feedId});

/// The production [RustFeedPort]: forwards straight to the generated
/// flutter_rust_bridge feed functions, each injectable for tests.
final class FfiRustFeedPort implements RustFeedPort {
  const FfiRustFeedPort({
    RustFeedSessionCapture session = feed_control.ffiFeedSession,
    RustFeedOpen open = feed_control.ffiOpenFeed,
    RustFeedWatch watch = feed_stream.ffiFeedUpdates,
    RustFeedMore more = feed_control.ffiLoadMore,
    RustFeedClose close = feed_control.ffiCloseFeed,
  })  : _session = session,
        _open = open,
        _watch = watch,
        _more = more,
        _close = close;

  final RustFeedSessionCapture _session;
  final RustFeedOpen _open;
  final RustFeedWatch _watch;
  final RustFeedMore _more;
  final RustFeedClose _close;

  @override
  Future<RustFeedAccountSession> captureSession(
    NostrPublicKeyHex? expectedAccount,
  ) async {
    final value = await _session(expectedAccountHex: expectedAccount?.value);
    return RustFeedAccountSession(
      account: expectedAccount,
      generation: RustNostrSessionGeneration.fromBridge(value),
    );
  }

  @override
  Future<RustFeedId> openFeed(
    FfiFeedSpec spec,
    RustFeedAccountSession session,
  ) async {
    final value = await _open(
      spec: spec,
      expectedAccountHex: session.account?.value,
      expectedSessionGeneration: session.generation.value,
    );
    return RustFeedId.parse(value);
  }

  @override
  Stream<FfiFeedUpdate> feedUpdates(RustFeedId feedId) {
    return _watch(feedId: feedId.value);
  }

  @override
  Future<bool> loadMore(RustFeedId feedId, {BigInt? olderThanSecs}) {
    return _more(feedId: feedId.value, olderThanSecs: olderThanSecs);
  }

  @override
  Future<void> closeFeed(RustFeedId feedId) {
    return _close(feedId: feedId.value);
  }
}
