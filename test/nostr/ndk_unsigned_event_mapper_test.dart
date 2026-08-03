import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_mapper.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('translates an unsigned domain event into an NDK event', () {
    final mapper = NdkNostrEventMapper(
      clock: () => DateTime.fromMillisecondsSinceEpoch(123000),
    );
    final event = NostrUnsignedEvent(
      kind: 5,
      tags: <List<String>>[],
      content: '',
    );

    final unsigned = mapper.toEvent(event, testViewerPublicKey);

    expect(unsigned.pubKey, testViewerPublicKey);
    expect(unsigned.kind, 5);
    expect(unsigned.tags, isEmpty);
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
