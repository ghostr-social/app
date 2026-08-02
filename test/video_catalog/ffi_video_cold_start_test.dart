import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_remote_source.dart';
import 'package:ghostr/src/rust/video/video.dart';

import '../support/ffi_video_fixture.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('builds a canonical Nostr post without a warm Dart snapshot', () async {
    final source = FfiVideoRemoteSource(
      gatewayBaseUrl: 'http://127.0.0.1:3000',
      snapshotLoader: () => [],
      loader: () async => [
        ffiVideo(
          id: 'cold',
          user: const FfiUserData(
            npub: testViewerNpub,
            name: 'Nora',
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
    expect(posts.single.media.remoteUrl, 'https://source.example/cold.mp4');
  });
}
