import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';

import '../support/nostr_reference.dart';
import '../support/propagation_delayed_nostr_client.dart';

void main() {
  test('serializes intents for revisions of one addressable target', () async {
    final gate = Completer<void>();
    final client = PropagationDelayedNostrClient(firstPublishGate: gate);
    final repository = NostrEngagementRepository(client);
    final oldRevision = nostrReference(identifier: 'clip');
    final newRevision = nostrReference(
      eventId: secondTestEventId,
      identifier: 'clip',
    );

    final like = repository.setLike(oldRevision, VideoLikeIntent.like);
    await client.firstPublishStarted.future;
    final unlike = repository.setLike(newRevision, VideoLikeIntent.unlike);
    await Future<void>.delayed(Duration.zero);
    expect(client.acceptedEvents, isEmpty);

    gate.complete();
    await like;
    final result = await unlike;

    expect(result.viewerHasLiked, isFalse);
    expect(client.acceptedEvents.map((event) => event.kind), [7, 5]);
  });
}
