import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_mapper.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('rejects an addressable NIP-71 event without a d identifier', () {
    final event = Nip01Event(
      id: testEventId,
      pubKey:
          '7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e',
      kind: 34236,
      content: 'Malformed addressable video',
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
