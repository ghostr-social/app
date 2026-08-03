import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('includes the address coordinate when liking an addressable video',
      () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final repository = NostrEngagementRepository(client);
    final video = nostrReference(
      kind: 34236,
      identifier: 'clip-1',
    );

    await repository.setLike(video, VideoLikeIntent.like);

    expect(
      client.events.single.tags,
      contains(equals(<String>[
        'a',
        '34236:$testCreatorPublicKey:clip-1',
      ])),
    );
  });
}
