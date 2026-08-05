import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/rust_nostr_event_mapper.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('creates the local unsigned NDK event used for signing', () {
    final mapper = RustNostrEventMapper(
      clock: () => DateTime.fromMillisecondsSinceEpoch(123000),
    );
    final event = NostrUnsignedEvent(
      kind: 5,
      tags: const <List<String>>[],
      content: 'delete',
    );

    final unsigned = mapper.toUnsignedEvent(
      event,
      NostrPublicKeyHex.parse(testViewerPublicKey),
    );

    expect(unsigned.pubKey, testViewerPublicKey);
    expect(unsigned.kind, 5);
    expect(unsigned.tags, isEmpty);
    expect(unsigned.content, 'delete');
    expect(unsigned.createdAt, 123);
    expect(
      unsigned.id,
      Nip01Utils.calculateEventIdSync(
        pubKey: unsigned.pubKey,
        createdAt: unsigned.createdAt,
        kind: unsigned.kind,
        tags: unsigned.tags,
        content: unsigned.content,
      ),
    );
  });
}
