import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';

import 'nostr_test_values.dart';

NostrEventRecord profileMetadataEvent(
  String content, {
  int kind = 0,
  int createdAt = 1700000000,
  String id = testEventId,
}) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: id,
      authorPublicKeyHex: testViewerPublicKey,
      kind: kind,
    ),
    tags: const [],
    content: content,
    createdAt: createdAt,
  );
}
