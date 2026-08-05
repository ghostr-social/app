import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/rust_nostr_event_client.dart';
import 'package:ghostr/src/rust/api/event_types.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_signed_event_broadcast_port.dart';

void main() {
  test('keeps the first unique matching Rust events up to the limit', () async {
    final first = _event(1);
    final sent = <FfiNostrEventFilter>[];
    final client = RustNostrEventClient(
      ndk: MockNdk(),
      broadcast: RecordingSignedEventBroadcastPort(),
      queries: RustNostrEventQueries(
        one: ({required filter}) async {
          sent.add(filter);
          return <FfiNostrEvent>[
            first,
            first,
            _event(2, kind: 6),
            _event(3),
            _event(4),
          ];
        },
      ),
    );

    final records = await client.query(
      NostrEventQuery(kinds: const <int>[7], limit: 2),
    );

    expect(sent.single.limit, 2);
    expect(records.map((event) => event.id.value), <String>[
      publishedEventId(1),
      publishedEventId(3),
    ]);
  });
}

FfiNostrEvent _event(int sequence, {int kind = 7}) {
  return FfiNostrEvent(
    id: publishedEventId(sequence),
    pubkey: testAuthorPublicKey,
    kind: kind,
    tags: const <List<String>>[],
    content: '+',
    createdAt: BigInt.from(sequence),
  );
}
