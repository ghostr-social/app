import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/rust_nostr_event_client.dart';
import 'package:ghostr/src/rust/api/event_types.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_signed_event_broadcast_port.dart';

void main() {
  test('applies a snapshotted batch policy to ordered Rust results', () async {
    final response = Completer<List<FfiNostrEvent>>();
    final sent = <List<FfiNostrEventFilter>>[];
    final client = RustNostrEventClient(
      ndk: MockNdk(),
      broadcast: RecordingSignedEventBroadcastPort(),
      queries: RustNostrEventQueries(
        batch: ({required filters}) {
          sent.add(filters);
          return response.future;
        },
      ),
    );
    final queries = <NostrEventQuery>[
      NostrEventQuery(kinds: const <int>[7], limit: 1),
      NostrEventQuery(
        kinds: const <int>[7],
        scope: NostrEventQueryScope.parse(
          eventTags: const <String>[testEventId],
        ),
        limit: 2,
      ),
    ];

    final pending = client.queryBatch(queries);
    queries
      ..clear()
      ..add(NostrEventQuery(kinds: const <int>[6]));
    final first = _event(1);
    response.complete(<FfiNostrEvent>[
      first,
      first,
      _event(2),
      _event(3, targeted: true),
      _event(4, targeted: true),
      _event(5, targeted: true),
    ]);

    expect(sent.single, hasLength(2));
    expect((await pending).map((event) => event.id.value), <String>[
      publishedEventId(1),
      publishedEventId(3),
      publishedEventId(4),
    ]);
  });
}

FfiNostrEvent _event(int sequence, {bool targeted = false}) {
  return FfiNostrEvent(
    id: publishedEventId(sequence),
    pubkey: testAuthorPublicKey,
    kind: 7,
    tags: targeted
        ? const <List<String>>[
            <String>['e', testEventId],
          ]
        : const <List<String>>[],
    content: '+',
    createdAt: BigInt.from(sequence),
  );
}
