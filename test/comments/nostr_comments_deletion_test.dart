import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/comments/data/nostr_comments_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('removes only comments deleted by their own author', () async {
    final deleted = publishedEventId(20);
    final retained = publishedEventId(21);
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.addAll([
        _comment(deleted, testFanPublicKey, 'Deleted'),
        _comment(retained, testAuthorPublicKey, 'Retained'),
        _deletion(publishedEventId(22), testFanPublicKey, deleted),
        _deletion(publishedEventId(23), testFanPublicKey, retained),
      ]);

    final comments = await NostrCommentsRepository(client).load(
      nostrReference(),
    );

    expect(comments.map((comment) => comment.content), ['Retained']);
    expect(client.requestCount, 2);
    expect(client.queryBatches.last, hasLength(1));
    expect(client.queryBatches.last.single.eventTags, hasLength(2));
    expect(client.queryBatches.last.single.limit, 500);
  });
}

NostrEventRecord _comment(String id, String author, String content) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: id,
      authorPublicKeyHex: author,
      kind: 1111,
    ),
    tags: const [
      ['E', testEventId],
    ],
    content: content,
    createdAt: 10,
  );
}

NostrEventRecord _deletion(String id, String author, String target) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: id,
      authorPublicKeyHex: author,
      kind: 5,
    ),
    tags: [
      ['e', target],
    ],
    content: 'deleted',
    createdAt: 10,
  );
}
