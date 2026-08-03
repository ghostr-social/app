import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_mapper.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

void main() {
  const mapper = NostrVideoEventMapper();

  test('maps a kind-1 note carrying a video link into a video post', () {
    final event = Nip01Event(
      id: testEventId,
      pubKey: testViewerPublicKey,
      kind: 1,
      createdAt: 1773302400,
      content: 'street show https://cdn.example/show.mp4 #street',
      tags: const [
        ['t', 'street'],
      ],
    );

    final post = mapper.map(event, null);

    expect(post.media.remoteUrl, 'https://cdn.example/show.mp4');
    expect(post.caption, 'street show #street');
    expect(post.hashtags, contains('street'));
    expect(post.nostrReference?.kind, 1);
    expect(post.nostrReference?.identifier, isNull);
  });

  test('rejects a kind-1 note with no playable video link', () {
    final event = Nip01Event(
      id: testEventId,
      pubKey: testViewerPublicKey,
      kind: 1,
      createdAt: 1773302400,
      content: 'just words, no video here',
      tags: const [],
    );

    expect(() => mapper.map(event, null), throwsA(isA<AppFailure>()));
  });
}
