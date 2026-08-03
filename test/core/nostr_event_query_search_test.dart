import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('a search term is normalized and never rejects events locally', () {
    final query = NostrEventQuery(kinds: const [21], search: ' ghost dance ');

    expect(query.search, 'ghost dance');
    expect(NostrEventQuery(kinds: const [21], search: '   ').search, isNull);
    expect(NostrEventQuery(kinds: const [21]).search, isNull);
    // NIP-50 matching is relay-defined; local revalidation stays structural.
    expect(query.matches(_record()), isTrue);
  });
}

NostrEventRecord _record() {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: testEventId,
      authorPublicKeyHex: testAuthorPublicKey,
      kind: 21,
    ),
    tags: const <List<String>>[],
    content: 'unrelated content',
    createdAt: 1,
  );
}
