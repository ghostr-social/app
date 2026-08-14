import 'package:ghostr/core/nostr/nostr_event_record.dart';

import 'nostr_test_values.dart';

NostrEventRecord viewerRepostEvent(
  int kind,
  String id,
  List<List<String>> tags,
  int createdAt,
) {
  return NostrUnsignedEvent(kind: kind, tags: tags, content: '').toRecord(
    id: id,
    authorPublicKeyHex: testViewerPublicKey,
    createdAt: createdAt,
  );
}
