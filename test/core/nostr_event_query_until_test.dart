import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('an until cutoff accepts events at or before it and rejects newer', () {
    final query = NostrEventQuery(kinds: const [21], until: 100);

    expect(query.matches(_record(createdAt: 100)), isTrue);
    expect(query.matches(_record(createdAt: 99)), isTrue);
    expect(query.matches(_record(createdAt: 101)), isFalse);
    expect(
      () => NostrEventQuery(kinds: const [21], until: -1),
      throwsFormatException,
    );
  });
}

NostrEventRecord _record({required int createdAt}) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: testEventId,
      authorPublicKeyHex: testAuthorPublicKey,
      kind: 21,
    ),
    tags: const <List<String>>[],
    content: '',
    createdAt: createdAt,
  );
}
