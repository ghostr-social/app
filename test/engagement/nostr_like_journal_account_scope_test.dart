import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';

import '../support/nostr_reference.dart';
import '../support/propagation_delayed_nostr_client.dart';

void main() {
  test('accepted like overlay remains scoped to its signing account', () async {
    final client = PropagationDelayedNostrClient();
    final repository = NostrEngagementRepository(client);
    final reference = nostrReference();

    await repository.setLike(reference, VideoLikeIntent.like);
    client.publicKeyHex = NostrPublicKeyHex.parse(testAuthorPublicKey);
    final otherAccount = await repository.load(reference);
    client.publicKeyHex = NostrPublicKeyHex.parse(testViewerPublicKey);
    final originalAccount = await repository.load(reference);

    expect(otherAccount.viewerHasLiked, isFalse);
    expect(otherAccount.likeCount, 0);
    expect(originalAccount.viewerHasLiked, isTrue);
    expect(originalAccount.likeCount, 1);
  });
}
