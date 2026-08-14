import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/reposts/data/nostr_repost_event_builder.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

import '../support/repost_samples.dart';

void main() {
  test('signed protection tag prevents embedding despite a false hint', () {
    final protected = repostablePost(protected: true).nostrReference!;
    final reference = NostrEventReference(
      eventId: protected.eventId,
      authorPublicKeyHex: protected.authorPublicKeyHex,
      kind: protected.kind,
      details: NostrEventReferenceDetails(
        signedEvent: protected.signedEvent,
        isProtected: false,
      ),
    );

    final event = buildRepostEvent(reference, 'wss://relay.example');

    expect(reference.isProtected, isTrue);
    expect(event.content, isEmpty);
  });
}
