import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_mapper.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('unions normalized t tags and caption hashtags into post hashtags', () {
    const publicKey =
        '7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e';
    final event = Nip01Event(
      id: testEventId,
      pubKey: publicKey,
      kind: 22,
      createdAt: 1773302400,
      content: 'Relay moves #Dance #Beats',
      tags: const [
        ['t', '#Nostr'],
        ['t', 'dance'],
        ['t', '  '],
        [
          'imeta',
          'url https://cdn.example/video.mp4',
          'm video/mp4',
        ],
      ],
    );

    final post = const NostrVideoEventMapper().map(event, null);

    expect(post.hashtags, ['nostr', 'dance', 'beats']);
  });
}
