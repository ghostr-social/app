import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

import 'nostr_test_values.dart';

export 'nostr_test_values.dart';

NostrEventReference nostrReference({
  String eventId = testEventId,
  String authorPublicKeyHex = testCreatorPublicKey,
  int kind = 22,
  String? identifier,
}) {
  return NostrEventReference(
    eventId: NostrEventId.parse(eventId),
    authorPublicKeyHex: NostrPublicKeyHex.parse(authorPublicKeyHex),
    kind: NostrEventKind.parse(kind),
    identifier:
        identifier == null ? null : NostrEventIdentifier.parse(identifier),
  );
}
