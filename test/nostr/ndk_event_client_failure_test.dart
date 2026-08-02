import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_client.dart';
import 'package:mocktail/mocktail.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('translates low-level NDK query and publish failures', () async {
    final ndk = MockNdk();
    final requests = MockRequests();
    final accounts = MockAccounts();
    final broadcast = MockBroadcast();
    when(() => ndk.requests).thenReturn(requests);
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.broadcast).thenReturn(broadcast);
    when(accounts.getPublicKey).thenReturn(testViewerPublicKey);
    when(
      () => requests.query(
        name: any(named: 'name'),
        filter: any(named: 'filter'),
        explicitRelays: any(named: 'explicitRelays'),
        timeout: any(named: 'timeout'),
      ),
    ).thenThrow(StateError('socket failed'));
    when(
      () => broadcast.broadcast(
        nostrEvent: any(named: 'nostrEvent'),
        specificRelays: any(named: 'specificRelays'),
      ),
    ).thenThrow(StateError('signer failed'));
    final client = NdkNostrEventClient(ndk: ndk, relays: const []);

    await expectLater(
      client.query(NostrEventQuery(kinds: [7])),
      throwsA(isA<AppFailure>()),
    );
    await expectLater(
      client.publish(NostrUnsignedEvent(kind: 7, tags: [], content: '+')),
      throwsA(isA<AppFailure>()),
    );
  });
}
