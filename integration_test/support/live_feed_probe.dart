import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/ffi_rust_feed_port.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_port.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import 'live_video_log.dart';

final class LiveFeedProbe implements RustFeedPort {
  LiveFeedProbe(this.log);
  final LiveVideoLog log;
  final RustFeedPort delegate = const FfiRustFeedPort();

  @override
  Future<RustFeedAccountSession> captureSession(
    NostrPublicKeyHex? expectedAccount,
  ) => delegate.captureSession(expectedAccount);

  @override
  Future<RustFeedId> openFeed(
    FfiFeedSpec spec,
    RustFeedAccountSession session,
  ) async {
    log.add('feed_open_started', {'spec': spec.toString()});
    final result = await delegate.openFeed(spec, session);
    log.add('feed_opened', {'feedId': result.value});
    return result;
  }

  @override
  Stream<FfiFeedUpdate> feedUpdates(RustFeedId feedId) =>
      delegate.feedUpdates(feedId).map((update) {
        log.add('feed_update', {
          'feedId': feedId.value,
          'stage': update.stage.name,
          'revision': update.revision.toString(),
          'postCount': update.posts.length,
          'firstEventIds': update.posts
              .take(5)
              .map((post) => post.eventId)
              .toList(),
        });
        return update;
      });

  @override
  Future<bool> loadMore(RustFeedId feedId, {BigInt? olderThanSecs}) {
    log.add('feed_load_more', {'feedId': feedId.value});
    return delegate.loadMore(feedId, olderThanSecs: olderThanSecs);
  }

  @override
  Future<void> closeFeed(RustFeedId feedId) => delegate.closeFeed(feedId);
}
