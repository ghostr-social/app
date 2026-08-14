import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/repost_samples.dart';

void main() {
  test('undo deletes every active viewer wrapper for the original', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    client.events.addAll([
      _wrapper(testEventId, secondTestEventId, 10),
      _wrapper(testEventId, publishedTestEventId, 11),
    ]);
    final repository = NostrVideoRepostRepository(
      client,
      relayHint: (_) async => 'wss://relay.example',
    );

    final updated = await repository.toggleRepost(
      repostablePost().withRepost(true),
    );

    final deletion = client.events.last;
    expect(deletion.kind.value, 5);
    expect(
      deletion.tagValues('e'),
      unorderedEquals([secondTestEventId, publishedTestEventId]),
    );
    expect(updated.viewerHasReposted, isFalse);
  });
}

NostrEventRecord _wrapper(String target, String id, int createdAt) {
  return NostrUnsignedEvent(
    kind: 6,
    tags: [
      ['e', target, 'wss://relay.example'],
      ['p', testCreatorPublicKey],
    ],
    content: '',
  ).toRecord(
    id: id,
    authorPublicKeyHex: testViewerPublicKey,
    createdAt: createdAt,
  );
}
