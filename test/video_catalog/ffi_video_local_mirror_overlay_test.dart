import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_remote_source.dart';
import 'package:ghostr/src/rust/video/video.dart';

import '../support/ffi_video_fixture.dart';
import '../support/nostr_test_values.dart';
import '../support/nostr_video_post_fixture.dart';

void main() {
  test('a native overlay retains canonical remote mirrors', () async {
    const primary = 'https://source.example/mirrored.mp4';
    const mirror = 'https://mirror.example/mirrored.mp4';
    final canonical = nostrVideoPost(const NostrVideoPostFixture(
      eventId: testEventId,
      mediaId: 'mirrored',
    )).withMedia(VideoMediaSource.remote(primary, fallbackUrls: [mirror]));
    final source = FfiVideoRemoteSource(
      snapshotLoader: () => [canonical],
      loader: () async => [
        ffiVideo(
          id: 'mirrored',
          user: const FfiUserData(),
          options: const FfiVideoFixtureOptions(
            localPath: '/cache/mirrored.mp4',
          ),
          event: ffiNostrEvent(identifier: 'mirrored'),
        ),
      ],
    );

    final media = (await source.loadRemoteFeed()).single.media;

    expect(media.localPath, isNull);
    expect(media.remoteUrls, [
      primary,
      mirror,
    ]);
  });
}
