import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_remote_source.dart';
import 'package:ghostr/src/rust/video/video.dart';

import '../support/ffi_video_fixture.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('builds a canonical Nostr post without a warm Dart snapshot', () async {
    final source = FfiVideoRemoteSource(
      snapshotLoader: () => [],
      loader: () async => [
        ffiVideo(
          id: 'cold',
          user: const FfiUserData(
            npub: testViewerNpub,
            name: 'Nora',
          ),
          options: const FfiVideoFixtureOptions(
            localPath: '/cache/cold.mp4',
            expectedDigest: _digest,
            fallbackUrls: ['https://mirror.example/cold.mp4'],
          ),
          event: ffiNostrEvent(eventId: secondTestEventId),
        ),
      ],
    );

    final posts = await source.loadRemoteFeed();

    expect(posts.single.id.value, secondTestEventId);
    expect(posts.single.nostrReference?.eventId.value, secondTestEventId);
    expect(posts.single.creator.displayName, 'Nora');
    expect(posts.single.caption, 'Relay dance');
    expect(posts.single.media.importPath, '/cache/cold.mp4');
    expect(posts.single.media.localPath, isNull);
    expect(posts.single.media.remoteUrl, 'https://source.example/cold.mp4');
    expect(posts.single.media.remoteUrls, [
      'https://source.example/cold.mp4',
      'https://mirror.example/cold.mp4',
    ]);
    expect(posts.single.media.expectedSha256?.value, _digest);
    expect(posts.single.media.cacheScope?.value, secondTestEventId);
  });
}

const _digest =
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
