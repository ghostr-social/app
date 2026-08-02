import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_mapper.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('uses a valid NIP-71 fallback when the primary URL is invalid', () {
    const publicKey =
        '7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e';
    final event = Nip01Event(
      id: testEventId,
      pubKey: publicKey,
      kind: 22,
      content: 'Fallback video',
      tags: const [
        [
          'imeta',
          'url malformed',
          'fallback https://mirror.example/video.mp4',
          'm video/mp4',
        ],
      ],
    );

    final post = const NostrVideoEventMapper().map(event, null);

    expect(post.media.remoteUrl, 'https://mirror.example/video.mp4');
  });
}
