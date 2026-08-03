import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('loads deletions for disjoint event and address reaction batches',
      () async {
    final reference = nostrReference(identifier: 'clip');
    final coordinate = '${reference.kind}:${reference.authorPublicKeyHex}:clip';
    final reactions = <NostrEventRecord>[
      for (var index = 1; index <= 300; index += 1)
        _reaction(index, ['e', reference.eventId]),
      for (var index = 301; index <= 600; index += 1)
        _reaction(index, ['a', coordinate]),
    ];
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.addAll([
        ...reactions,
        for (var index = 0; index < reactions.length; index += 1)
          _deletion(index + 1001, reactions[index].id),
      ]);

    final engagement = await NostrEngagementRepository(client).load(reference);

    expect(engagement.likeCount, 0);
    final deletionQueries = client.queryBatches
        .skip(client.queryBatches.length - 2)
        .expand((batch) => batch)
        .toList(growable: false);
    expect(client.requestCount, 6);
    expect(deletionQueries.map((query) => query.eventTags.length), [500, 100]);
    expect(deletionQueries.every((query) => query.limit == 500), isTrue);
  });
}

NostrEventRecord _reaction(int sequence, List<String> targetTag) {
  return _event(sequence, 7, [targetTag], '+');
}

NostrEventRecord _deletion(int sequence, String reactionId) {
  return _event(
      sequence,
      5,
      [
        ['e', reactionId],
      ],
      'deleted');
}

NostrEventRecord _event(
  int sequence,
  int kind,
  List<List<String>> tags,
  String content,
) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: publishedEventId(sequence),
      authorPublicKeyHex: testFanPublicKey,
      kind: kind,
    ),
    tags: tags,
    content: content,
    createdAt: sequence,
  );
}
