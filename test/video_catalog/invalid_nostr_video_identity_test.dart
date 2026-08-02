import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_mapper.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('translates an invalid Nostr video identity into an app failure', () {
    final event = Nip01Event(
      id: 'not-an-event-id',
      pubKey: testCreatorPublicKey,
      kind: 21,
      content: 'Malformed identity',
      tags: const [
        ['imeta', 'url https://cdn.example/video.mp4', 'm video/mp4'],
      ],
    );

    expect(
      () => const NostrVideoEventMapper().map(event, null),
      throwsA(isA<AppFailure>()),
    );
  });
}
