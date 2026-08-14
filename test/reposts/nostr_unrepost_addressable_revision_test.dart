import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/repost_samples.dart';

void main() {
  test('undo deletes an addressable wrapper for an older revision', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    client.events.add(_oldRevisionWrapper());
    final repository = NostrVideoRepostRepository(
      client,
      relayHint: (_) async => null,
    );
    final post = repostablePost(
      kind: 34235,
      identifier: 'clip',
    ).withRepost(true);

    await repository.toggleRepost(post);

    expect(client.events.last.kind.value, 5);
    expect(client.events.last.tagValues('e'), contains(publishedTestEventId));
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
