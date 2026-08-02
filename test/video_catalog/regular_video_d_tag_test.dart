import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_mapper.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('ignores a d tag on a regular NIP-71 video event', () {
    final event = Nip01Event(
      id: testEventId,
      pubKey:
          '7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e',
      kind: 22,
      content: 'Regular video',
      tags: const [
        ['d', 'must-not-be-an-address'],
        ['imeta', 'url https://cdn.example/video.mp4', 'm video/mp4'],
      ],
    );

    final post = const NostrVideoEventMapper().map(event, null);

    expect(post.nostrReference?.identifier, isNull);
  });
}
