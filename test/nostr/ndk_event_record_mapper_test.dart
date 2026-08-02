import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_mapper.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('translates a signed NDK event into a domain record', () {
    const mapper = NdkNostrEventMapper();
    final event = Nip01Event(
      id: testEventId,
      pubKey: testAuthorPublicKey,
      kind: 7,
      tags: const [
        ['e', secondTestEventId],
      ],
      content: '+',
      createdAt: 12,
    );

    final record = mapper.toRecord(event);

    expect(record.id.value, testEventId);
    expect(record.authorPublicKeyHex.value, testAuthorPublicKey);
    expect(record.kind.value, 7);
    expect(record.tagValues('e'), <String>[secondTestEventId]);
  });
}
