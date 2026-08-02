import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('loads an addressable reaction that uses only its a coordinate',
      () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    client.events.add(NostrEventRecord(
      identity: NostrEventIdentity.parse(
        id: secondTestEventId,
        authorPublicKeyHex: testFanPublicKey,
        kind: 7,
      ),
      tags: [
        ['a', '34236:$testCreatorPublicKey:clip-1'],
        ['p', testCreatorPublicKey],
      ],
      content: '+',
      createdAt: 1,
    ));
    final repository = NostrEngagementRepository(client);
    final video = nostrReference(
      eventId: publishedTestEventId,
      kind: 34236,
      identifier: 'clip-1',
    );

    final engagement = await repository.load(video);

    expect(engagement.likeCount, 1);
  });
}
