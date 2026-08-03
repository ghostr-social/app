import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';

import '../support/nostr_reference.dart';
import '../support/propagation_delayed_nostr_client.dart';

void main() {
  test('observed reaction and deletion retire pending lookup targets',
      () async {
    final client = PropagationDelayedNostrClient();
    final repository = NostrEngagementRepository(client);
    final reference = nostrReference();

    await repository.setLike(reference, VideoLikeIntent.like);
    await repository.setLike(reference, VideoLikeIntent.unlike);
    final reaction = client.acceptedEvents.first;
    client.propagateAcceptedEvents();
    expect((await repository.load(reference)).viewerHasLiked, isFalse);

    client.events
      ..clear()
      ..add(reaction);
    client.queries.clear();
    client.queryBatches.clear();
    client.requestCount = 0;
    final afterPropagation = await repository.load(reference);

    expect(afterPropagation.viewerHasLiked, isFalse);
    expect(client.requestCount, 2);

    final reliked = await repository.setLike(reference, VideoLikeIntent.like);
    expect(reliked.viewerHasLiked, isTrue);
    expect((await repository.load(reference)).viewerHasLiked, isTrue);
  });
}
