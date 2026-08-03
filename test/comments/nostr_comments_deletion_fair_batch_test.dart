import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/comments/data/nostr_comments_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('one comment cannot crowd out another comment deletion', () async {
    final first = publishedEventId(10);
    final second = publishedEventId(11);
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.addAll([
        _event(first, 1111, testFanPublicKey, testEventId),
        _event(second, 1111, testAuthorPublicKey, testEventId),
        _event(publishedEventId(12), 5, testFanPublicKey, first),
        _event(publishedEventId(13), 5, testFanPublicKey, first),
        _event(publishedEventId(14), 5, testAuthorPublicKey, second),
      ]);

    final comments = await NostrCommentsRepository(client).load(
      nostrReference(),
    );

    expect(comments, isEmpty);
    expect(client.requestCount, 2);
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
      [kind == 5 ? 'e' : 'E', target],
    ],
    content: kind == 5 ? 'deleted' : 'Comment',
    createdAt: 10,
  );
}
