import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';

import 'nostr_test_values.dart';

class FakeNostrEventClient implements NostrEventClient {
  FakeNostrEventClient({required String publicKeyHex})
      : publicKeyHex = NostrPublicKeyHex.parse(publicKeyHex);

  @override
  final NostrPublicKeyHex publicKeyHex;
  final List<NostrEventRecord> events = <NostrEventRecord>[];

  @override
  Future<NostrEventId> publish(NostrUnsignedEvent event) async {
    final id = NostrEventId.parse(publishedEventId(events.length + 1));
    events.add(event.toRecord(
      id: id,
      authorPublicKeyHex: publicKeyHex,
      createdAt: 1700000000 + events.length,
    ));
    return id;
  }

  @override
  Future<List<NostrEventRecord>> query(NostrEventQuery query) async {
    return events.where(query.matches).take(query.limit).toList();
  }
}
