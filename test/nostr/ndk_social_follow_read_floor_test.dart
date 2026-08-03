import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_social.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('stale follow reads cannot undo an accepted replacement', () async {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final follows = MockFollows();
    final broadcast = MockBroadcast();
    final response = MockNdkBroadcastResponse();
    final signer = MockEventSigner();
    final config = MockNdkConfig();
    final cache = MockCacheManager();
    var reads = 0;
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.follows).thenReturn(follows);
    when(() => ndk.broadcast).thenReturn(broadcast);
    when(() => ndk.config).thenReturn(config);
    when(() => config.cache).thenReturn(cache);
    when(accounts.getPublicKey).thenReturn(testViewerPublicKey);
    when(accounts.getLoggedAccount).thenReturn(Account(
      type: AccountType.privateKey,
      pubkey: testViewerPublicKey,
      signer: signer,
    ));
    stubEventSigner(signer, testViewerPublicKey);
    when(() => follows.getContactList(
          testViewerPublicKey,
          forceRefresh: true,
        )).thenAnswer((_) async => _contacts(10));
    when(() => follows.getContactList(testViewerPublicKey))
        .thenAnswer((_) async {
      reads += 1;
      return reads == 2 ? _contacts(30) : _contacts(10);
    });
    when(() => response.broadcastDoneFuture)
        .thenAnswer((_) async => successfulRelayBroadcast());
    when(() => broadcast.broadcast(
          nostrEvent: any(named: 'nostrEvent'),
          specificRelays: any(named: 'specificRelays'),
          customSigner: signer,
          saveToCache: false,
        )).thenReturn(response);
    when(() => cache.saveEvent(any())).thenThrow(StateError('cache failed'));
    when(() => cache.saveContactList(any()))
        .thenThrow(StateError('cache failed'));
    final social = NdkNostrSocial(
      ndk: ndk,
      relays: [RelayUrl.parse('wss://relay.example')],
      clock: () => DateTime.fromMillisecondsSinceEpoch(20000),
    );
    final target = ProfileId.parse(Nip19.encodePubKey(testFanPublicKey));

    expect(await social.toggleFollow(target), isTrue);
    expect(await social.loadFollowedProfiles(), {target});
    expect(await social.loadFollowedProfiles(), isEmpty);
    expect(await social.loadFollowedProfiles(), isEmpty);
  });
}

ContactList _contacts(int createdAt) {
  return ContactList(pubKey: testViewerPublicKey, contacts: <String>[])
    ..createdAt = createdAt;
}
