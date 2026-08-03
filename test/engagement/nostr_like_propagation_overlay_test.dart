import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';

import '../support/nostr_reference.dart';
import '../support/propagation_delayed_nostr_client.dart';

void main() {
  test('accepted like overlays single and batch reads before propagation',
      () async {
    final client = PropagationDelayedNostrClient();
    final repository = NostrEngagementRepository(client);
    final reference = nostrReference();

    await repository.setLike(reference, VideoLikeIntent.like);
    final loaded = await repository.load(reference);
    final batch = await repository.loadBatch([reference]);

    expect(loaded.likeCount, 1);
    expect(loaded.viewerHasLiked, isTrue);
    expect(batch[reference.eventId]!.likeCount, 1);
    expect(batch[reference.eventId]!.viewerHasLiked, isTrue);
    expect(client.acceptedEvents, hasLength(1));
  });
}
