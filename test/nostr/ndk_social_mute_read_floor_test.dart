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

  test('stale mute reads cannot undo an accepted replacement', () async {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final lists = MockLists();
    final broadcast = MockBroadcast();
    final response = MockNdkBroadcastResponse();
    final signer = MockEventSigner();
    final config = MockNdkConfig();
    final cache = MockCacheManager();
    var reads = 0;
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.lists).thenReturn(lists);
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
    when(() => signer.encryptNip44(
          plaintext: any(named: 'plaintext'),
          recipientPubKey: testViewerPublicKey,
        )).thenAnswer((_) async => 'encrypted');
    when(() => lists.getSingleNip51List(Nip51List.kMute)).thenAnswer((_) async {
      reads += 1;
      return _mute(reads == 3 ? 30 : 10);
    });
    when(() => lists.getSingleNip51List(
          Nip51List.kMute,
          forceRefresh: true,
        )).thenAnswer((_) async => _mute(10));
    when(() => response.broadcastDoneFuture)
        .thenAnswer((_) async => successfulRelayBroadcast());
    when(() => broadcast.broadcast(
          nostrEvent: any(named: 'nostrEvent'),
          specificRelays: any(named: 'specificRelays'),
          customSigner: signer,
          saveToCache: false,
        )).thenReturn(response);
    when(() => cache.saveEvent(any())).thenThrow(StateError('cache failed'));
    final social = NdkNostrSocial(
      ndk: ndk,
      relays: [RelayUrl.parse('wss://relay.example')],
      clock: () => DateTime.fromMillisecondsSinceEpoch(20000),
    );
    final target = ProfileId.parse(Nip19.encodePubKey(testFanPublicKey));

    expect(await social.toggleBlock(target), isTrue);
    expect(await social.loadBlockedProfiles(), {target});
    expect(await social.loadBlockedProfiles(), isEmpty);
    expect(await social.loadBlockedProfiles(), isEmpty);
  });
}

Nip51List _mute(int createdAt) {
  return Nip51List(
    pubKey: testViewerPublicKey,
    kind: Nip51List.kMute,
    createdAt: createdAt,
    elements: <Nip51ListElement>[],
  );
}
