import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/comments/data/nostr_comments_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('a malformed reply id does not poison the comment batch', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.addAll(<NostrEventRecord>[
        _comment(publishedEventId(1), 'Malformed', const [
          ['e', 'not-an-event-id'],
          ['k', '1111'],
        ]),
        _comment(publishedEventId(2), 'Valid', const []),
      ]);

    final comments = await NostrCommentsRepository(client).load(
      nostrReference(),
    );

    expect(comments.map((comment) => comment.content), ['Malformed', 'Valid']);
    expect(comments.first.parentCommentId, isNull);
  });
}

NostrEventRecord _comment(
  String id,
  String content,
  List<List<String>> replyTags,
) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: id,
      authorPublicKeyHex: testFanPublicKey,
      kind: 1111,
    ),
    tags: <List<String>>[
      const <String>['E', testEventId],
      ...replyTags,
    ],
    content: content,
    createdAt: 10,
  );
}
