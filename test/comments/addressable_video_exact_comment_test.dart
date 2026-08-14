import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/data/nostr_comments_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('publishes comments to the exact address coordinate', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final reference = nostrReference(
      kind: 34236,
      identifier: 'clip',
      publishedIdentifier: ' clip ',
    );

    await NostrCommentsRepository(
      client,
    ).publish(reference: reference, content: 'Exact clip');

    expect(client.events.single.tagValues('A'), [
      '34236:$testCreatorPublicKey: clip ',
    ]);
    expect(client.events.single.tagValues('a'), [
      '34236:$testCreatorPublicKey: clip ',
    ]);
  });
}
