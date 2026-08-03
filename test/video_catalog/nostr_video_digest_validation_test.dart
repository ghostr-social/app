import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_mapper.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('rejects Nostr video media with a malformed advertised digest', () {
    final event = Nip01Event(
      id: testEventId,
      pubKey: testViewerPublicKey,
      kind: 34236,
      createdAt: 1773302400,
      content: 'Untrusted video',
      tags: const [
        ['d', 'untrusted'],
        [
          'imeta',
          'url https://cdn.example/video.mp4',
          'm video/mp4',
          'x not-a-sha256',
        ],
      ],
    );

    expect(
      () => const NostrVideoEventMapper().map(event, null),
      throwsA(isA<AppFailure>()),
    );
  });
}
