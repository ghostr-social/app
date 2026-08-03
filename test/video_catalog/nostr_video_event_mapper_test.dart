import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_mapper.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('maps a NIP-71 short-video event into a referenced video post', () {
    const publicKey =
        '7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e';
    final event = Nip01Event(
      id: testEventId,
      pubKey: publicKey,
      kind: 34236,
      createdAt: 1773302400,
      content: 'A real Nostr short',
      tags: const [
        ['d', 'clip-1'],
        ['title', 'Original sound'],
        [
          'imeta',
          'url https://cdn.example/video.mp4',
          'm video/mp4',
          'x AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA',
          'image https://cdn.example/poster.jpg',
          'fallback https://mirror.example/video.mp4',
        ],
      ],
    );
    final metadata = Metadata(
      pubKey: publicKey,
      displayName: 'Nora Relay',
      picture: 'https://cdn.example/nora.jpg',
    );

    final post = const NostrVideoEventMapper().map(event, metadata);

    expect(post.id, testEventId);
    expect(post.creator.id,
        'npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg');
    expect(post.caption, 'A real Nostr short');
    expect(post.media.remoteUrl, 'https://cdn.example/video.mp4');
    expect(post.media.remoteUrls, [
      'https://cdn.example/video.mp4',
      'https://mirror.example/video.mp4',
    ]);
    expect(
      post.media.expectedSha256?.value,
      'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
    );
    expect(post.media.cacheScope?.value, testEventId);
    expect(post.nostrReference?.eventId, testEventId);
    expect(post.nostrReference?.authorPublicKeyHex, publicKey);
    expect(post.nostrReference?.kind, 34236);
    expect(post.nostrReference?.identifier, 'clip-1');
  });
}
