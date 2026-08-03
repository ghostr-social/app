import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('a busy reaction target cannot starve another batched target', () async {
    final first = nostrReference();
    final second = nostrReference(eventId: secondTestEventId);
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.addAll([
        for (var sequence = 1; sequence <= 1000; sequence += 1)
          _reaction(sequence, first.eventId),
        _reaction(1001, second.eventId),
      ]);

    final engagement = await NostrEngagementRepository(client).loadBatch([
      first,
      second,
    ]);

    expect(engagement[first.eventId]!.likeCount, 1);
    expect(engagement[second.eventId]!.likeCount, 1);
    expect(client.requestCount, 3);
    expect(
      client.queryBatches.every((batch) => batch.length <= 20),
      isTrue,
    );
  });
}

NostrEventRecord _reaction(int sequence, String target) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: publishedEventId(sequence),
      authorPublicKeyHex: testFanPublicKey,
      kind: 7,
    ),
    tags: [
      ['e', target],
    ],
    content: '+',
    createdAt: 2000 - sequence,
  );
}
