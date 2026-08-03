import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_remote_source.dart';
import 'package:ghostr/src/rust/video/video.dart';

import '../support/ffi_video_fixture.dart';
import '../support/nostr_test_values.dart';
import '../support/nostr_video_post_fixture.dart';

void main() {
  test('matches the exact media attachment within a shared Nostr event',
      () async {
    const url = 'https://source.example/genuine.mp4';
    final canonical = nostrVideoPost(const NostrVideoPostFixture(
      eventId: testEventId,
      mediaId: 'genuine',
    )).withMedia(VideoMediaSource.withExpectedSha256(
      VideoMediaSource.remote(url),
      _genuineDigest,
    ));
    final source = FfiVideoRemoteSource(
      snapshotLoader: () => [canonical],
      loader: () async => [
        _native(_wrongDigest, 'https://source.example/wrong.mp4', '/wrong.mp4'),
        _native(_genuineDigest, url, '/genuine.mp4'),
      ],
    );

    final post = (await source.loadRemoteFeed()).first;

    expect(post.media.localPath, isNull);
    expect(post.media.remoteUrl, url);
  });
}

FfiVideoDownload _native(String id, String url, String path) {
  return ffiVideo(
    id: id,
    user: const FfiUserData(),
    options: FfiVideoFixtureOptions(localPath: path, mediaUrl: url),
    event: ffiNostrEvent(eventId: testEventId, identifier: 'genuine'),
  );
}

const _genuineDigest =
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
const _wrongDigest =
    'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
