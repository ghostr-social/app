import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('loads reactions for the exact published address coordinate', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    client.events.add(
      NostrUnsignedEvent(
        kind: 7,
        tags: [
          ['a', '34236:$testCreatorPublicKey: clip '],
        ],
        content: '+',
      ).toRecord(
        id: secondTestEventId,
        authorPublicKeyHex: testFanPublicKey,
        createdAt: 1,
      ),
    );
    final reference = nostrReference(
      kind: 34236,
      identifier: 'clip',
      publishedIdentifier: ' clip ',
    );

    final engagement = await NostrEngagementRepository(client).load(reference);

    expect(engagement.likeCount, 1);
  });
}
