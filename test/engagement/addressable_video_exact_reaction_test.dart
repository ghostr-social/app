import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('likes the exact published address coordinate', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final reference = nostrReference(
      kind: 34236,
      identifier: 'clip',
      publishedIdentifier: ' clip ',
    );

    await NostrEngagementRepository(
      client,
    ).setLike(reference, VideoLikeIntent.like);

    expect(client.events.single.tagValues('a'), [
      '34236:$testCreatorPublicKey: clip ',
    ]);
  });
}
