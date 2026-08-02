import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_remote_source.dart';
import 'package:ghostr/src/rust/video/video.dart';

import '../support/ffi_video_fixture.dart';

void main() {
  test('preserves HLS delivery from native fallback inventory', () async {
    const url = 'https://media.example/video.m3u8';
    final source = FfiVideoRemoteSource(
      gatewayBaseUrl: 'http://127.0.0.1:3000',
      snapshotLoader: () => const [],
      loader: () async => [
        ffiVideo(
          id: 'hls',
          user: const FfiUserData(),
          options: const FfiVideoFixtureOptions(
            mediaUrl: url,
            delivery: FfiVideoDelivery.hls,
          ),
        ),
      ],
    );

    final post = (await source.loadRemoteFeed()).single;

    expect(post.media.remoteUrl, url);
    expect(post.media.remoteDelivery, VideoMediaDelivery.hls);
    expect(post.media.canCacheAsSingleFile, isFalse);
    expect(post.media.fallbackUrls, isEmpty);
  });
}
