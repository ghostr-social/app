import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/signed_nostr_event_json.dart';

final class NostrEventReferenceDetails {
  const NostrEventReferenceDetails({
    this.identifier,
    this.publishedIdentifier,
    this.signedEvent,
    this.isProtected = false,
  });

  final NostrEventIdentifier? identifier;
  final NostrEventIdentifier? publishedIdentifier;
  final SignedNostrEventJson? signedEvent;
  final bool isProtected;
}

class NostrEventReference {
  const NostrEventReference({
    required this.eventId,
    required this.authorPublicKeyHex,
    required this.kind,
    this.details,
  });

  final NostrEventId eventId;
  final NostrPublicKeyHex authorPublicKeyHex;
  final NostrEventKind kind;
  final NostrEventReferenceDetails? details;

  NostrEventIdentifier? get identifier => details?.identifier;
  NostrEventIdentifier? get publishedIdentifier => details?.publishedIdentifier;
  NostrEventIdentifier? get coordinateIdentifier =>
      publishedIdentifier ?? identifier;
  SignedNostrEventJson? get signedEvent => details?.signedEvent;
  bool get isProtected =>
      (details?.isProtected ?? false) || (signedEvent?.isProtected ?? false);
}
