import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('deletes every active viewer reaction when unliking', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.addAll([
        _reaction(secondTestEventId, testViewerPublicKey, 1),
        _reaction(publishedTestEventId, testViewerPublicKey, 2),
        _reaction(publishedEventId(4), testFanPublicKey, 3),
      ]);
    final repository = NostrEngagementRepository(client);

    final engagement = await repository.setLike(
      nostrReference(),
      VideoLikeIntent.unlike,
    );

    expect(engagement.likeCount, 1);
    expect(engagement.viewerHasLiked, isFalse);
    expect(client.events.last.kind, 5);
    expect(client.events.last.tagValues('e').toSet(), {
      secondTestEventId,
      publishedTestEventId,
    });
  });
}

NostrEventRecord _reaction(String id, String author, int createdAt) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: id,
      authorPublicKeyHex: author,
      kind: 7,
    ),
    tags: const [
      ['e', testEventId],
    ],
    content: '+',
    createdAt: createdAt,
  );
}
