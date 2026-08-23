import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/ffi_rust_feed_port.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_port.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

final class WarpFeedRustProbe implements RustFeedPort {
  WarpFeedRustProbe([RustFeedPort? delegate])
    : _delegate = delegate ?? const FfiRustFeedPort();

  final RustFeedPort _delegate;
  final List<String> _evidence = [];

  String get evidence => _evidence.join(',');

  @override
  Future<RustFeedAccountSession> captureSession(
    NostrPublicKeyHex? expectedAccount,
  ) async {
    final session = await _delegate.captureSession(expectedAccount);
    _evidence.add('session=${session.generation.value}');
    return session;
  }

  @override
  Future<RustFeedId> openFeed(
    FfiFeedSpec spec,
    RustFeedAccountSession session,
  ) async {
    final feed = await _delegate.openFeed(spec, session);
    _evidence.add('open=${feed.value}');
    return feed;
  }

  @override
  Stream<FfiFeedUpdate> feedUpdates(RustFeedId feedId) {
    return _delegate.feedUpdates(feedId).map((update) {
      _evidence.add(
        '${update.stage.name}:${update.revision}:${update.posts.length}',
      );
      return update;
    });
  }

  @override
  Future<bool> loadMore(RustFeedId feedId, {BigInt? olderThanSecs}) {
    return _delegate.loadMore(feedId, olderThanSecs: olderThanSecs);
  }

  @override
  Future<void> closeFeed(RustFeedId feedId) {
    return _delegate.closeFeed(feedId);
  }
}
