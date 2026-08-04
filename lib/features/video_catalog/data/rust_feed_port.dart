import 'package:ghostr/src/rust/api/feed_types.dart';

/// Discovery feeds served by the Rust engine (plan §5), behind a port
/// so the FFI plugin stays at the boundary and tests inject a fake.
abstract interface class RustFeedPort {
  /// Opens the feed named by [spec] and returns its handle.
  Future<String> openFeed(FfiFeedSpec spec);

  /// Full snapshots of one open feed: a baseline immediately, then one
  /// per visible-list revision until the feed closes.
  Stream<FfiFeedUpdate> feedUpdates(String feedId);

  /// Requests one older page; false once the feed is exhausted.
  Future<bool> loadMore(String feedId, {BigInt? olderThanSecs});

  /// Closes the feed: posts drop and update streams end.
  Future<void> closeFeed(String feedId);
}
