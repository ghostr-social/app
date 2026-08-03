import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/comments/data/nostr_comments_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('a busy comment thread cannot starve another batched thread', () async {
    final first = nostrReference();
    final second = nostrReference(eventId: secondTestEventId);
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.addAll([
        for (var sequence = 1; sequence <= 1000; sequence += 1)
          _comment(sequence, first.eventId),
        _comment(1001, second.eventId),
      ]);

    final comments = await NostrCommentsRepository(client).loadBatch([
      first,
      second,
    ]);

    expect(comments[first.eventId], hasLength(500));
    expect(comments[second.eventId]!.single.content, 'Comment 1001');
    expect(client.requestCount, 2);
    expect(
      client.queryBatches.every((batch) => batch.length <= 20),
      isTrue,
    );
  });
}

NostrEventRecord _comment(int sequence, String target) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: publishedEventId(sequence),
      authorPublicKeyHex: testFanPublicKey,
      kind: 1111,
    ),
    tags: [
      ['E', target],
    ],
    content: 'Comment $sequence',
    createdAt: 2000 - sequence,
  );
}
