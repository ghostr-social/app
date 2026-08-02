import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('publishes and removes a NIP-25 like for a video event', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.add(_otherReaction());
    final repository = NostrEngagementRepository(client);
    final video = nostrReference();

    final liked = await repository.toggleLike(video);

    expect(liked.likeCount, 2);
    expect(liked.viewerHasLiked, isTrue);
    expect(client.events.last.kind, 7);
    expect(client.events.last.content, '+');
    expect(
        client.events.last.tags,
        containsAll(<List<String>>[
          ['e', testEventId],
          ['p', testCreatorPublicKey],
          ['k', '22'],
        ]));

    final unliked = await repository.toggleLike(video);

    expect(unliked.likeCount, 1);
    expect(unliked.viewerHasLiked, isFalse);
    expect(client.events.last.kind, 5);
    expect(
        client.events.last.tags,
        containsAll(<List<String>>[
          ['e', publishedEventId(2)],
          ['k', '7'],
        ]));
  });
}

NostrEventRecord _otherReaction() {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: secondTestEventId,
      authorPublicKeyHex: testAuthorPublicKey,
      kind: 7,
    ),
    tags: [
      ['e', testEventId],
    ],
    content: '+',
    createdAt: 1,
  );
}
