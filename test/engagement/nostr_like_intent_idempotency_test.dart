import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';

import '../support/nostr_reference.dart';
import '../support/propagation_delayed_nostr_client.dart';

void main() {
  test('repeated desired-like intent does not publish duplicate reactions',
      () async {
    final client = PropagationDelayedNostrClient();
    final repository = NostrEngagementRepository(client);
    final reference = nostrReference();

    await repository.setLike(reference, VideoLikeIntent.like);
    final repeated = await repository.setLike(
      reference,
      VideoLikeIntent.like,
    );

    expect(repeated.viewerHasLiked, isTrue);
    expect(repeated.likeCount, 1);
    expect(client.acceptedEvents, hasLength(1));
  });
}
