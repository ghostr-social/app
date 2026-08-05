import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/nostr/rust_nostr_event_mapper.dart';
import 'package:ghostr/src/rust/api/event_types.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('maps every verified Rust event field into a domain record', () {
    const mapper = RustNostrEventMapper();
    final event = _event(BigInt.from(1700000123));

    final record = mapper.toRecord(event);

    expect(record.id.value, testEventId);
    expect(record.authorPublicKeyHex.value, testAuthorPublicKey);
    expect(record.kind.value, 7);
    expect(record.tags.toRaw(), <List<String>>[
      <String>['A', '34235:author:clip', 'root'],
    ]);
    expect(record.content, '+');
    expect(record.createdAt, 1700000123);
  });

  test('rejects a Rust timestamp that cannot round-trip into Dart', () {
    const mapper = RustNostrEventMapper();

    expect(
      () => mapper.toRecord(_event(BigInt.one << 80)),
      throwsA(isA<FormatException>()),
    );
  });
}

FfiNostrEvent _event(BigInt createdAt) {
  return FfiNostrEvent(
    id: testEventId,
    pubkey: testAuthorPublicKey,
    kind: 7,
    tags: const <List<String>>[
      <String>['A', '34235:author:clip', 'root'],
    ],
    content: '+',
    createdAt: createdAt,
  );
}
