import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/domain/signed_event_broadcast_port.dart';
import 'package:ghostr/features/social/domain/signed_nostr_event_json.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_social.dart';
import 'package:mocktail/mocktail.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('follow reads query the active author kind 3 with limit one', () async {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final events = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    when(() => ndk.accounts).thenReturn(accounts);
    when(accounts.getPublicKey).thenReturn(testViewerPublicKey);
    final social = NdkNostrSocial(
      ndk: ndk,
      eventClient: events,
      broadcast: _NoopBroadcastPort(),
    );

    expect(await social.loadFollowedProfiles(), isEmpty);

    final query = events.queries.single;
    expect(query.kinds.map((kind) => kind.value), [3]);
    expect(query.authors.map((author) => author.value), [
      testViewerPublicKey,
    ]);
    expect(query.limit, 1);
  });
}

final class _NoopBroadcastPort implements SignedEventBroadcastPort {
  @override
  Future<void> broadcast(SignedNostrEventJson event) async {}
}
