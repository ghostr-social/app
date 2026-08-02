import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';

abstract interface class NostrEventClient {
  NostrPublicKeyHex get publicKeyHex;

  Future<List<NostrEventRecord>> query(NostrEventQuery query);

  Future<NostrEventId> publish(NostrUnsignedEvent event);
}
