import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('uses an explicitly supplied timestamp for a test event record', () {
    final event = NostrUnsignedEvent(kind: 7, tags: [], content: '+');

    final record = event.toRecord(
      id: testEventId,
      authorPublicKeyHex: testAuthorPublicKey,
      createdAt: 42,
    );

    expect(record.createdAt, 42);
  });
}
