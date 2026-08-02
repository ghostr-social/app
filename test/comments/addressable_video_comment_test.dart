import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/data/nostr_comments_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('scopes an addressable video comment with NIP-22 A tags', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final repository = NostrCommentsRepository(client);
    final video = nostrReference(
      kind: 34236,
      identifier: 'clip-1',
    );

    await repository.publish(reference: video, content: 'Nice clip');

    expect(
        client.events.single.tags,
        containsAll(<List<String>>[
          ['A', '34236:$testCreatorPublicKey:clip-1'],
          ['a', '34236:$testCreatorPublicKey:clip-1'],
          ['e', testEventId],
          ['K', '34236'],
          ['k', '34236'],
        ]));
  });
}
