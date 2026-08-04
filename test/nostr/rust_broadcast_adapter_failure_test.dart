import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/nostr/rust_broadcast_adapter.dart';
import 'package:ghostr/platform/nostr/signed_nostr_event_json.dart';

import '../support/signed_event_fixture.dart';

void main() {
  test('reports an engine broadcast error the way relay rejection reads',
      () async {
    final adapter = RustBroadcastAdapter(
      send: ({required String signedEventJson}) async {
        throw StateError('engine offline');
      },
    );

    await expectLater(
      adapter.broadcast(encodeSignedNostrEvent(signedTestEvent())),
      throwsA(isA<AppFailure>().having(
        (failure) => failure.message,
        'message',
        'No Nostr relay accepted the event.',
      )),
    );
  });
}
