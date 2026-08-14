import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/signed_nostr_event_json.dart';

abstract interface class NostrEventClient {
  NostrPublicKeyHex get publicKeyHex;

  Future<List<NostrEventRecord>> query(NostrEventQuery query);

  Future<List<NostrEventRecord>> queryBatch(List<NostrEventQuery> queries);

  Future<NostrEventId> publish(
    NostrUnsignedEvent event, {
    required NostrPublicKeyHex expectedAuthor,
  });
}

final class NostrEventPublication {
  const NostrEventPublication({required this.id, this.signedEvent});

  final NostrEventId id;
  final SignedNostrEventJson? signedEvent;
}

abstract interface class SignedNostrEventPublisher {
  Future<NostrEventPublication> publishSigned(
    NostrUnsignedEvent event, {
    required NostrPublicKeyHex expectedAuthor,
  });
}
