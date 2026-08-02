import 'package:ghostr/core/nostr/nostr_event_identity.dart';

class NostrEventReference {
  const NostrEventReference({
    required this.eventId,
    required this.authorPublicKeyHex,
    required this.kind,
    this.identifier,
  });

  final NostrEventId eventId;
  final NostrPublicKeyHex authorPublicKeyHex;
  final NostrEventKind kind;
  final NostrEventIdentifier? identifier;
}
