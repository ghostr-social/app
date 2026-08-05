import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/rust_nostr_event_client.dart';
import 'package:ghostr/src/rust/api/event_types.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_signed_event_broadcast_port.dart';

void main() {
  test('rejects the whole result set when one Rust event is malformed',
      () async {
    final client = RustNostrEventClient(
      ndk: MockNdk(),
      broadcast: RecordingSignedEventBroadcastPort(),
      queries: RustNostrEventQueries(
        one: ({required filter}) async {
          return <FfiNostrEvent>[
            _event(testEventId),
            _event('not-an-event-id'),
          ];
        },
      ),
    );

    await expectLater(
      client.query(NostrEventQuery(kinds: const <int>[7])),
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          'Could not read from Nostr relays.',
        ),
      ),
    );
  });
}

FfiNostrEvent _event(String id) {
  return FfiNostrEvent(
    id: id,
    pubkey: testAuthorPublicKey,
    kind: 7,
    tags: const <List<String>>[],
    content: '+',
    createdAt: BigInt.one,
  );
}
