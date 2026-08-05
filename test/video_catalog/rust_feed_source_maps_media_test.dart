import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  test('maps media urls, digest, metadata, and the engine cache scope',
      () async {
    const digest =
        'dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd';
    final port = FakeRustFeedPort(updates: [
      rustFeedBaseline(),
      rustFeedUpdate(revision: 1, posts: [
        rustFeedPost(
          postId: 'engine-post-id',
          media: rustFeedMedia(
            urls: const [
              'https://cdn.example/clip.mp4',
              'https://mirror.example/clip.mp4',
            ],
            sha256: digest,
            sizeBytes: 2048,
            durationMs: 9000,
          ),
        ),
      ]),
    ]);
    final source = RustFeedRemoteSource(port: port);

    final posts = await source.loadRemoteFeed(searchQuery: 'ghost');

    final media = posts.single.media;
    expect(media.remoteUrl, 'https://cdn.example/clip.mp4');
    expect(media.fallbackUrls, const ['https://mirror.example/clip.mp4']);
    expect(media.remoteDelivery, VideoMediaDelivery.progressive);
    expect(media.expectedSha256?.value, digest);
    // The engine's gateway post id becomes the cache scope so focus
    // updates address the same Rust store entry
    // (ffi_focus_item_media_mapper.dart prefers the scope).
    expect(media.cacheScope?.value, 'engine-post-id');
    expect(media.mediaMetadata.sizeBytes, 2048);
    expect(media.mediaMetadata.durationMs, 9000);
  });

  test('maps hls delivery through to the domain media source', () async {
    final port = FakeRustFeedPort(updates: [
      rustFeedBaseline(),
      rustFeedUpdate(revision: 1, posts: [
        rustFeedPost(
          media: rustFeedMedia(
            urls: const ['https://cdn.example/clip.m3u8'],
            delivery: FfiMediaDelivery.hls,
          ),
        ),
      ]),
    ]);
    final source = RustFeedRemoteSource(port: port);

    final posts = await source.loadRemoteFeed(searchQuery: 'ghost');

    expect(posts.single.media.remoteDelivery, VideoMediaDelivery.hls);
  });
}
