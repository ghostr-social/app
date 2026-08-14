import 'package:ghostr/core/nostr/nostr_event_record.dart';

import 'nostr_test_values.dart';

NostrEventRecord relayListEvent({
  required String id,
  required int createdAt,
  required List<List<String>> tags,
  String author = testCreatorPublicKey,
}) {
  return NostrUnsignedEvent(
    kind: 10002,
    tags: tags,
    content: '',
  ).toRecord(id: id, authorPublicKeyHex: author, createdAt: createdAt);
}
