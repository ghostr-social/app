import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';

import '../support/nostr_event_fixture.dart';
import '../support/nostr_reference.dart';
import '../support/propagation_delayed_nostr_client.dart';

void main() {
  test('relay omission cannot undo a like without valid deletion evidence',
      () async {
    final client = PropagationDelayedNostrClient();
    final repository = NostrEngagementRepository(client);
    final reference = nostrReference();

    await repository.setLike(reference, VideoLikeIntent.like);
    final reactionId = client.acceptedEvents.single.id;
    client.propagateAcceptedEvents();
    expect((await repository.load(reference)).viewerHasLiked, isTrue);

    client.events.clear();
    expect((await repository.load(reference)).viewerHasLiked, isTrue);

    client.events.add(nostrEventFixture(
      publishedEventId(900),
      testViewerPublicKey,
      5,
      [
        ['e', reactionId],
      ],
    ));

    final deleted = await repository.load(reference);
    expect(deleted.viewerHasLiked, isFalse);
    expect(deleted.likeCount, 0);

    client.events.clear();
    client.requestCount = 0;
    expect((await repository.load(reference)).viewerHasLiked, isFalse);
    expect(client.requestCount, 2);
  });
}
