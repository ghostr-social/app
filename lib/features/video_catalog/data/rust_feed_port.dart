import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_identity.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

/// Discovery feeds served by the Rust engine, behind a port so generated
/// FFI stays at the boundary and tests can inject a fake.
abstract interface class RustFeedPort {
  /// Captures the active native account session before a request waits.
  Future<RustFeedAccountSession> captureSession(
    NostrPublicKeyHex? expectedAccount,
  );

  /// Opens the feed named by [spec] and returns its handle.
  Future<RustFeedId> openFeed(
    FfiFeedSpec spec,
    RustFeedAccountSession session,
  );

  /// Full snapshots of one open feed: a baseline immediately, then one
  /// per visible-list revision until the feed closes.
  Stream<FfiFeedUpdate> feedUpdates(RustFeedId feedId);

  /// Requests one older page; false once the feed is exhausted.
  Future<bool> loadMore(RustFeedId feedId, {BigInt? olderThanSecs});

  /// Closes the feed: posts drop and update streams end.
  Future<void> closeFeed(RustFeedId feedId);
}
