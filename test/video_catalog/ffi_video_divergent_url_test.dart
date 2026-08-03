import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_remote_source.dart';
import 'package:ghostr/src/rust/video/video.dart';

import '../support/ffi_video_fixture.dart';
import '../support/nostr_test_values.dart';
import '../support/nostr_video_post_fixture.dart';

void main() {
  test('rejects an overlay whose native URL fields disagree', () async {
    const url = 'https://source.example/video.mp4';
    final canonical = nostrVideoPost(const NostrVideoPostFixture(
      eventId: testEventId,
      mediaId: 'dance',
    )).withMedia(VideoMediaSource.remote(url));
    final source = FfiVideoRemoteSource(
      snapshotLoader: () => [canonical],
      loader: () async => [
        _native('/cache/untrusted.mp4', 'https://attacker.example/video.mp4'),
        _native('/cache/genuine.mp4', url),
      ],
    );

    final post = (await source.loadRemoteFeed()).first;

    expect(post.media.localPath, isNull);
    expect(post.media.remoteUrl, url);
    expect(post.media.remoteUrls, contains(url));
    expect(post.media.remoteUrls,
        isNot(contains('https://attacker.example/video.mp4')));
  });
}

FfiVideoDownload _native(String path, String nostrUrl) {
  return ffiVideo(
    id: 'dance',
    user: const FfiUserData(),
    options: FfiVideoFixtureOptions(
      localPath: path,
      mediaUrl: 'https://source.example/video.mp4',
      nostrUrl: nostrUrl,
    ),
  );
}
