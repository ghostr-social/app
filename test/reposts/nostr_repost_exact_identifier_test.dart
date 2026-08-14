import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/reposts/data/nostr_repost_event_builder.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('addressable repost uses the exact published identifier', () {
    final reference = NostrEventReference(
      eventId: NostrEventId.parse(testEventId),
      authorPublicKeyHex: NostrPublicKeyHex.parse(testCreatorPublicKey),
      kind: NostrEventKind.parse(34235),
      details: NostrEventReferenceDetails(
        identifier: NostrEventIdentifier.parse('clip'),
        publishedIdentifier: NostrEventIdentifier.published(' clip '),
        isProtected: true,
      ),
    );

    final event = buildRepostEvent(reference, null);

    expect(
      event.tags.last,
      orderedEquals(['a', '34235:$testCreatorPublicKey: clip ']),
    );
  });
}
