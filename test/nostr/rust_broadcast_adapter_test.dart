import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/nostr/rust_broadcast_adapter.dart';
import 'package:ghostr/platform/nostr/signed_nostr_event_json.dart';

import '../support/signed_event_fixture.dart';

void main() {
  test('hands the signed event JSON to the engine untouched', () async {
    final sent = <String>[];
    final adapter = RustBroadcastAdapter(
      send: ({required String signedEventJson}) async {
        sent.add(signedEventJson);
      },
    );
    final json = encodeSignedNostrEvent(signedTestEvent(kind: 10000));

    await adapter.broadcast(json);

    expect(sent, [json]);
    expect(decodeSignedNostrEvent(sent.single).sig, testEventSignature);
  });
}
