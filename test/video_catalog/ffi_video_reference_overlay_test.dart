import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_remote_source.dart';
import 'package:ghostr/src/rust/video/video.dart';

import '../support/ffi_video_fixture.dart';
import '../support/nostr_test_values.dart';
import '../support/nostr_video_post_fixture.dart';

void main() {
  test('rejects a same-ID native row with mismatched event coordinates',
      () async {
    final canonical = nostrVideoPost(const NostrVideoPostFixture(
      eventId: testEventId,
      mediaId: 'shared',
    ));
    final source = FfiVideoRemoteSource(
      snapshotLoader: () => [canonical],
      loader: () async => [
        _native(testAuthorPublicKey, '/wrong.mp4'),
        _native(testCreatorPublicKey, '/genuine.mp4'),
      ],
    );

    final post = (await source.loadRemoteFeed()).first;

    expect(post.media.localPath, isNull);
    expect(post.media.remoteUrl, 'https://source.example/shared.mp4');
  });
}

FfiVideoDownload _native(String author, String path) {
  return ffiVideo(
    id: 'shared',
    user: const FfiUserData(),
    options: FfiVideoFixtureOptions(localPath: path),
    event: ffiNostrEvent(
      eventId: testEventId,
      authorPublicKeyHex: author,
      identifier: 'shared',
    ),
  );
}
