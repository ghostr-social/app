import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_cache_identity.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_mapper.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('scopes unpinned cache identities to the Nostr event revision', () {
    const mapper = NostrVideoEventMapper();

    final first = mapper.map(_event(testEventId), null).media;
    final second = mapper.map(_event(secondTestEventId), null).media;

    expect(first.cacheScope?.value, testEventId);
    expect(second.cacheScope?.value, secondTestEventId);
    expect(first.cacheStorageIdentity, isNot(second.cacheStorageIdentity));
    expect(first.cacheJobIdentity, isNot(second.cacheJobIdentity));
  });
}

Nip01Event _event(String id) {
  return Nip01Event(
    id: id,
    pubKey: testCreatorPublicKey,
    kind: 22,
    createdAt: 20,
    content: 'Mutable media',
    tags: const [
      [
        'imeta',
        'url https://media.example/mutable.mp4',
        'm video/mp4',
      ],
    ],
  );
}
