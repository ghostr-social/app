import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('one reaction cannot crowd out another reaction deletion', () async {
    final first = publishedEventId(10);
    final second = publishedEventId(11);
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.addAll([
        _event(first, 7, testFanPublicKey, testEventId),
        _event(second, 7, testAuthorPublicKey, secondTestEventId),
        _event(publishedEventId(12), 5, testFanPublicKey, first),
        _event(publishedEventId(13), 5, testFanPublicKey, first),
        _event(publishedEventId(14), 5, testAuthorPublicKey, second),
      ]);

    final engagement = await NostrEngagementRepository(client).loadBatch([
      nostrReference(),
      nostrReference(eventId: secondTestEventId),
    ]);

    expect(engagement.values.every((item) => item.likeCount == 0), isTrue);
    expect(client.requestCount, 3);
  });
}

NostrEventRecord _event(String id, int kind, String author, String target) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: id,
      authorPublicKeyHex: author,
      kind: kind,
    ),
    tags: [
      ['e', target],
    ],
    content: kind == 5 ? 'deleted' : '+',
    createdAt: 10,
  );
}
