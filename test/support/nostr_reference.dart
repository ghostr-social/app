import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

import 'nostr_test_values.dart';

export 'nostr_test_values.dart';

NostrEventReference nostrReference({
  String eventId = testEventId,
  int kind = 22,
  String? identifier,
  String? publishedIdentifier,
}) {
  return NostrEventReference(
    eventId: NostrEventId.parse(eventId),
    authorPublicKeyHex: NostrPublicKeyHex.parse(testCreatorPublicKey),
    kind: NostrEventKind.parse(kind),
    details: NostrEventReferenceDetails(
      identifier: identifier == null
          ? null
          : NostrEventIdentifier.parse(identifier),
      publishedIdentifier: publishedIdentifier == null
          ? null
          : NostrEventIdentifier.published(publishedIdentifier),
    ),
  );
}

NostrEventReference nostrReferenceForAuthor(
  String authorPublicKeyHex, {
  String eventId = testEventId,
  int kind = 22,
  String? identifier,
}) {
  return NostrEventReference(
    eventId: NostrEventId.parse(eventId),
    authorPublicKeyHex: NostrPublicKeyHex.parse(authorPublicKeyHex),
    kind: NostrEventKind.parse(kind),
    details: NostrEventReferenceDetails(
      identifier: identifier == null
          ? null
          : NostrEventIdentifier.parse(identifier),
    ),
  );
}
