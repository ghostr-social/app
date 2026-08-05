import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/rust_nostr_event_client.dart';

import '../support/ndk_mocks.dart';
import '../support/recording_signed_event_broadcast_port.dart';

void main() {
  test('translates a Rust query failure at the event-client boundary',
      () async {
    final client = RustNostrEventClient(
      ndk: MockNdk(),
      broadcast: RecordingSignedEventBroadcastPort(),
      queries: RustNostrEventQueries(
        one: ({required filter}) async {
          throw StateError('engine unavailable');
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
