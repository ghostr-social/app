import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/comments/data/nostr_comments_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('loads comments for the exact published address coordinate', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    client.events.add(
      NostrUnsignedEvent(
        kind: 1111,
        tags: [
          ['A', '34236:$testCreatorPublicKey: clip '],
        ],
        content: 'Exact clip',
      ).toRecord(
        id: secondTestEventId,
        authorPublicKeyHex: testFanPublicKey,
        createdAt: 1,
      ),
    );
    final reference = nostrReference(
      kind: 34236,
      identifier: 'clip',
      publishedIdentifier: ' clip ',
    );

    final comments = await NostrCommentsRepository(client).load(reference);

    expect(comments.single.content, 'Exact clip');
  });
}
