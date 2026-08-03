import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/comments/data/nostr_comments_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('loads regular and addressable comment threads in two batches',
      () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final regular = nostrReference();
    final addressable = nostrReference(
      eventId: secondTestEventId,
      kind: 34236,
      identifier: 'clip-2',
    );
    final coordinate =
        '${addressable.kind}:${addressable.authorPublicKeyHex}:clip-2';
    final parentId = publishedEventId(30);
    client.events.addAll([
      _comment(parentId, 'First', 10, [
        ['E', regular.eventId],
      ]),
      _comment(publishedEventId(31), 'Reply', 20, [
        ['E', regular.eventId],
        ['e', parentId],
        ['k', '1111'],
      ]),
      _comment(publishedEventId(32), 'Addressable', 15, [
        ['A', coordinate],
      ]),
      _comment(publishedEventId(33), '   ', 25, [
        ['A', coordinate],
      ]),
    ]);

    final comments = await NostrCommentsRepository(client).loadBatch([
      regular,
      addressable,
    ]);

    expect(comments[regular.eventId]!.map((item) => item.content), [
      'First',
      'Reply',
    ]);
    expect(comments[regular.eventId]!.last.parentCommentId, parentId);
    expect(comments[addressable.eventId]!.single.content, 'Addressable');
    expect(client.requestCount, 3);
    expect(
        client.queryBatches.take(2).map((batch) {
          return batch.single.tagFilters.single.name;
        }),
        [
          'E',
          'A',
        ]);
    expect(client.queryBatches.last, hasLength(2));
    expect(
      client.queryBatches.last.every((query) => query.limit == 500),
      isTrue,
    );
  });
}

NostrEventRecord _comment(
  String id,
  String content,
  int createdAt,
  List<List<String>> tags,
) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: id,
      authorPublicKeyHex: testFanPublicKey,
      kind: 1111,
    ),
    tags: tags,
    content: content,
    createdAt: createdAt,
  );
}
