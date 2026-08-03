import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_deletion_lookup.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('loads author-valid deletion IDs for one target group', () async {
    final target = _event(testEventId, testAuthorPublicKey, 1);
    final deletion =
        _event(publishedEventId(2), testAuthorPublicKey, 5, tags: const [
      ['e', testEventId],
    ]);
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.add(deletion);

    final deleted = await loadAuthorValidNostrDeletionIds(client, [target]);

    expect(deleted, {testEventId});
  });
}

NostrEventRecord _event(
  String id,
  String author,
  int kind, {
  List<List<String>> tags = const [],
}) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: id,
      authorPublicKeyHex: author,
      kind: kind,
    ),
    tags: tags,
    content: '',
    createdAt: 10,
  );
}
