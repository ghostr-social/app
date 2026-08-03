import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_remote_source.dart';
import 'package:ghostr/src/rust/video/video.dart';

import '../support/ffi_video_fixture.dart';
import '../support/nostr_test_values.dart';
import '../support/nostr_video_post_fixture.dart';

void main() {
  test('maps native bytes as a non-playable import candidate', () async {
    final canonical = nostrVideoPost(const NostrVideoPostFixture(
      eventId: testEventId,
      mediaId: 'owned',
    ));
    final source = FfiVideoRemoteSource(
      snapshotLoader: () => [canonical],
      loader: () async => [
        ffiVideo(
          id: 'owned',
          user: const FfiUserData(),
          options: const FfiVideoFixtureOptions(
            localPath: '/native/warm.mp4',
          ),
          event: ffiNostrEvent(identifier: 'owned'),
        ),
      ],
    );

    final media = (await source.loadRemoteFeed()).single.media;

    expect(media, isA<ImportableVideoMediaSource>());
    expect(media.importPath, '/native/warm.mp4');
    expect(media.localPath, isNull);
    expect(media.remoteUrl, 'https://source.example/owned.mp4');
  });
}
