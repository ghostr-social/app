import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import 'fake_rust_feed_port.dart';
import 'nostr_test_values.dart';
import 'rust_feed_fixture_options.dart';

export 'rust_feed_fixture_options.dart';

FfiFeedCreator rustFeedCreator({
  String pubkey = testCreatorPublicKey,
  String displayName = 'Nora Relay',
  String handle = '@norarelay',
  String? avatarUrl,
}) {
  return FfiFeedCreator(
    pubkey: pubkey,
    displayName: displayName,
    handle: handle,
    avatarUrl: avatarUrl,
  );
}

FfiFeedMedia rustFeedMedia({
  List<String> urls = const ['https://cdn.example/clip.mp4'],
  FfiMediaDelivery delivery = FfiMediaDelivery.progressive,
  RustFeedMediaDetails details = const RustFeedMediaDetails(),
}) {
  return FfiFeedMedia(
    urls: urls,
    delivery: delivery,
    sha256: details.sha256,
    sizeBytes: details.sizeBytes == null
        ? null
        : BigInt.from(details.sizeBytes!),
    durationMs: details.durationMs == null
        ? null
        : BigInt.from(details.durationMs!),
  );
}

FfiFeedPost rustFeedPost({
  String eventId = testEventId,
  int eventKind = 22,
  int createdAt = 1754000000,
  RustFeedPostDetails details = const RustFeedPostDetails(),
}) {
  return FfiFeedPost(
    postId: details.postId,
    eventId: eventId,
    eventKind: eventKind,
    identifier: details.identifier,
    createdAt: BigInt.from(createdAt),
    feedSortAt: BigInt.from(createdAt),
    isProtected: false,
    caption: details.caption,
    title: details.title,
    hashtags: details.hashtags,
    creator: details.creator ?? rustFeedCreator(),
    media: details.media ?? rustFeedMedia(),
  );
}

FfiFeedUpdate rustFeedUpdate({
  String feedId = '7',
  int revision = 0,
  FfiFeedStage stage = FfiFeedStage.settled,
  List<FfiFeedPost> posts = const <FfiFeedPost>[],
}) {
  return FfiFeedUpdate(
    feedId: feedId,
    revision: BigInt.from(revision),
    stage: stage,
    posts: posts,
  );
}

/// The snapshot every subscription opens with: the feed exists, its
/// first page is still in flight (rust/src/api/feed_updates_stream.rs).
FfiFeedUpdate rustFeedBaseline() => rustFeedUpdate(stage: FfiFeedStage.loading);

/// A signed-in Rust discovery source whose first page is exactly [posts].
RemoteVideoSource rustFeedSourceServing(List<FfiFeedPost> posts) {
  return RustFeedRemoteSource(
    port: FakeRustFeedPort(
      updates: <FfiFeedUpdate>[rustFeedUpdate(revision: 1, posts: posts)],
    ),
    viewer: () => NostrPublicKeyHex.parse(testViewerPublicKey),
  );
}
