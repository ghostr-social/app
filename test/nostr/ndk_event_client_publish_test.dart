import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_client.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/entities.dart' show RelayBroadcastResponse;
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('publishes a mapped event to the configured relays', () async {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final broadcast = MockBroadcast();
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.broadcast).thenReturn(broadcast);
    when(accounts.getPublicKey).thenReturn(testViewerPublicKey);
    when(
      () => broadcast.broadcast(
        nostrEvent: any(named: 'nostrEvent'),
        specificRelays: any(named: 'specificRelays'),
      ),
    ).thenAnswer((invocation) {
      final event = invocation.namedArguments[#nostrEvent] as Nip01Event;
      return NdkBroadcastResponse(
        publishEvent: event,
        broadcastDoneStream: Stream.value([
          RelayBroadcastResponse(
            relayUrl: 'wss://relay.example',
            broadcastSuccessful: true,
          ),
        ]),
      );
    });
    final client = NdkNostrEventClient(
      ndk: ndk,
      relays: [RelayUrl.parse('wss://relay.example')],
    );

    final id = await client.publish(
      NostrUnsignedEvent(kind: 7, tags: [], content: '+'),
    );

    expect(id, isNotEmpty);
    final call = verify(
      () => broadcast.broadcast(
        nostrEvent: captureAny(named: 'nostrEvent'),
        specificRelays: captureAny(named: 'specificRelays'),
      ),
    );
    expect((call.captured[0] as Nip01Event).pubKey, testViewerPublicKey);
    expect(call.captured[1], ['wss://relay.example']);
  });
}
