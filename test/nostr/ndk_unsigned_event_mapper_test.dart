import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_mapper.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('translates an unsigned domain event into an NDK event', () {
    const mapper = NdkNostrEventMapper();
    final event = NostrUnsignedEvent(
      kind: 5,
      tags: <List<String>>[],
      content: '',
    );

    final unsigned = mapper.toEvent(event, testViewerPublicKey);

    expect(unsigned.pubKey, testViewerPublicKey);
    expect(unsigned.kind, 5);
    expect(unsigned.tags, isEmpty);
  });
}
