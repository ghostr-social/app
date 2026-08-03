import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_remote_source.dart';
import 'package:ghostr/src/rust/video/video.dart';

import '../support/ffi_video_fixture.dart';
import '../support/nostr_test_values.dart';
import '../support/nostr_video_post_fixture.dart';

void main() {
  test('preserves canonical mirrors in a remote native overlay', () async {
    const primary = 'https://source.example/mirrored.mp4';
    const mirrors = [
      'https://mirror1.example/mirrored.mp4',
      'https://mirror2.example/mirrored.mp4',
      'https://mirror3.example/mirrored.mp4',
      'https://mirror4.example/mirrored.mp4',
    ];
    final canonical = nostrVideoPost(const NostrVideoPostFixture(
      eventId: testEventId,
      mediaId: 'mirrored',
    )).withMedia(VideoMediaSource.remote(primary, fallbackUrls: mirrors));
    final source = FfiVideoRemoteSource(
      snapshotLoader: () => [canonical],
      loader: () async => [
        ffiVideo(
          id: 'mirrored',
          user: const FfiUserData(),
          event: ffiNostrEvent(identifier: 'mirrored'),
        ),
      ],
    );

    final media = (await source.loadRemoteFeed()).single.media;

    expect(media.remoteUrls, [
      primary,
      ...mirrors,
    ]);
    expect(media.cacheSourceUrls, [primary, ...mirrors]);
  });
}
