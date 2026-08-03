import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_event_fixture.dart';
import '../support/nostr_reference.dart';

void main() {
  test('loads regular video engagement in one reaction and deletion batch',
      () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final firstReaction = publishedEventId(10);
    final secondReaction = publishedEventId(11);
    client.events.addAll([
      nostrEventFixture(firstReaction, testViewerPublicKey, 7, [
        ['e', testEventId],
      ]),
      nostrEventFixture(secondReaction, testFanPublicKey, 7, [
        ['e', secondTestEventId],
      ]),
      nostrEventFixture(publishedEventId(12), testFanPublicKey, 5, [
        ['e', secondReaction],
      ]),
      nostrEventFixture(publishedEventId(13), testAuthorPublicKey, 5, [
        ['e', firstReaction],
      ]),
    ]);
    final repository = NostrEngagementRepository(client);

    final engagements = await repository.loadBatch([
      nostrReference(eventId: testEventId),
      nostrReference(eventId: secondTestEventId),
    ]);

    expect(engagements[testEventId]!.likeCount, 1);
    expect(engagements[testEventId]!.viewerHasLiked, isTrue);
    expect(engagements[secondTestEventId]!.likeCount, 0);
    expect(engagements[secondTestEventId]!.viewerHasLiked, isFalse);
    expect(client.requestCount, 4);
    expect(client.queryBatches.first, hasLength(2));
    expect(
      client.queryBatches.first.every((query) => query.authors.isNotEmpty),
      isTrue,
    );
    expect(
      client.queryBatches[1].every((query) => query.authors.isEmpty),
      isTrue,
    );
    expect(
        client.queryBatches.first.every((query) => query.limit == 500), isTrue);
    expect(client.queryBatches[2], hasLength(1));
    expect(client.queryBatches[2].single.authors, [
      NostrPublicKeyHex.parse(testViewerPublicKey),
    ]);
    expect(client.queryBatches.last, hasLength(2));
  });

  test('batches addressable reactions without counting overlap twice',
      () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final first = nostrReference(identifier: 'clip-1');
    final second = nostrReference(
      eventId: secondTestEventId,
      identifier: 'clip-2',
    );
    final firstCoordinate = '${first.kind}:${first.authorPublicKeyHex}:clip-1';
    final secondCoordinate =
        '${second.kind}:${second.authorPublicKeyHex}:clip-2';
    client.events.addAll([
      nostrEventFixture(publishedEventId(20), testFanPublicKey, 7, [
        ['e', first.eventId],
        ['a', firstCoordinate],
      ]),
      nostrEventFixture(publishedEventId(21), testAuthorPublicKey, 7, [
        ['a', secondCoordinate],
      ]),
    ]);
    final engagements = await NostrEngagementRepository(client).loadBatch([
      first,
      second,
    ]);

    expect(engagements[first.eventId]!.likeCount, 1);
    expect(engagements[second.eventId]!.likeCount, 1);
    expect(client.requestCount, 5);
    expect(
        client.queryBatches[2].map((query) {
          return query.tagFilters.single.values.single;
        }).toSet(),
        {
          firstCoordinate,
          secondCoordinate,
        });
  });
}
