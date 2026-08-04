import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import 'fake_rust_feed_port.dart';
import 'nostr_test_values.dart';

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
  String delivery = 'progressive',
  String? sha256,
  int? sizeBytes,
  int? durationMs,
}) {
  return FfiFeedMedia(
    urls: urls,
    delivery: delivery,
    sha256: sha256,
    sizeBytes: sizeBytes == null ? null : BigInt.from(sizeBytes),
    durationMs: durationMs == null ? null : BigInt.from(durationMs),
  );
}

FfiFeedPost rustFeedPost({
  String postId = 'a1b2c3',
  String eventId = testEventId,
  int eventKind = 22,
  String? identifier,
  int createdAt = 1754000000,
  String caption = 'A relay-side banger',
  List<String> hashtags = const <String>[],
  FfiFeedCreator? creator,
  FfiFeedMedia? media,
}) {
  return FfiFeedPost(
    postId: postId,
    eventId: eventId,
    eventKind: eventKind,
    identifier: identifier,
    createdAt: BigInt.from(createdAt),
    caption: caption,
    hashtags: hashtags,
    creator: creator ?? rustFeedCreator(),
    media: media ?? rustFeedMedia(),
  );
}

FfiFeedUpdate rustFeedUpdate({
  String feedId = '7',
  int revision = 0,
  List<FfiFeedPost> posts = const <FfiFeedPost>[],
}) {
  return FfiFeedUpdate(
    feedId: feedId,
    revision: BigInt.from(revision),
    posts: posts,
  );
}

/// A signed-in Rust discovery source whose first page is exactly
/// [posts] — what the app runs on with FeedPipelineMode.rust.
RemoteVideoSource rustFeedSourceServing(List<FfiFeedPost> posts) {
  return RustFeedRemoteSource(
    port: FakeRustFeedPort(
      updates: <FfiFeedUpdate>[rustFeedUpdate(revision: 1, posts: posts)],
    ),
    viewer: () => NostrPublicKeyHex.parse(testViewerPublicKey),
  );
}
