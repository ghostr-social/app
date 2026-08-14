import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/repost_samples.dart';

void main() {
  test('hydrates an addressable repost made for an older revision', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    client.events.add(_oldRevisionWrapper());
    final repository = NostrVideoRepostRepository(
      client,
      relayHint: (_) async => null,
      hydrationTimeout: const Duration(seconds: 10),
    );

    final post = (await repository.hydrateAll([
      repostablePost(kind: 34235, identifier: 'clip'),
    ])).single;

    expect(post.viewerHasReposted, isTrue);
    expect(
      client.queries.expand((query) => query.tagFilters),
      contains(isA<NostrTagFilter>().having((tag) => tag.name, 'name', 'a')),
    );
  });
}

NostrEventRecord _oldRevisionWrapper() {
  return NostrUnsignedEvent(
    kind: 16,
    tags: [
      ['e', secondTestEventId],
      ['a', '34235:$testCreatorPublicKey:clip'],
    ],
    content: '',
  ).toRecord(
    id: publishedTestEventId,
    authorPublicKeyHex: testViewerPublicKey,
    createdAt: 10,
  );
}
