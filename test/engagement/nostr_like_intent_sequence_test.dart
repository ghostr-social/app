import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';

import '../support/nostr_reference.dart';
import '../support/propagation_delayed_nostr_client.dart';

void main() {
  test('like then unlike stays correct while relay reads remain stale',
      () async {
    final client = PropagationDelayedNostrClient();
    final repository = NostrEngagementRepository(client);
    final reference = nostrReference();

    final liked = await repository.setLike(reference, VideoLikeIntent.like);
    final unliked = await repository.setLike(
      reference,
      VideoLikeIntent.unlike,
    );
    final loaded = await repository.load(reference);

    expect(liked.viewerHasLiked, isTrue);
    expect(unliked.likeCount, 0);
    expect(unliked.viewerHasLiked, isFalse);
    expect(loaded.viewerHasLiked, isFalse);
    expect(client.acceptedEvents.map((event) => event.kind), [7, 5]);
    expect(client.acceptedEvents.last.tagValues('e'), [
      client.acceptedEvents.first.id,
    ]);
  });
}
