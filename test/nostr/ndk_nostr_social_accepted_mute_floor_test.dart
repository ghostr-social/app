import 'dart:async';

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

  test('an accepted mute floor survives an account switch during reread',
      () async {
    var account = testViewerPublicKey;
    var cachedReads = 0;
    final secondRead = Completer<Nip51List?>();
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final lists = MockLists();
    final broadcast = MockBroadcast();
    final response = MockNdkBroadcastResponse();
    final signer = MockEventSigner();
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.lists).thenReturn(lists);
    when(() => ndk.broadcast).thenReturn(broadcast);
    when(accounts.getPublicKey).thenAnswer((_) => account);
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
    when(() => lists.getSingleNip51List(Nip51List.kMute)).thenAnswer((_) {
      if (cachedReads++ == 0) return Future<Nip51List?>.value();
      return secondRead.future;
    });
    when(() => lists.getSingleNip51List(
          Nip51List.kMute,
          forceRefresh: true,
        )).thenAnswer((_) async => null);
    when(() => response.broadcastDoneFuture)
        .thenAnswer((_) async => successfulRelayBroadcast());
    when(() => broadcast.broadcast(
          nostrEvent: any(named: 'nostrEvent'),
          specificRelays: any(named: 'specificRelays'),
          customSigner: signer,
          saveToCache: false,
        )).thenReturn(response);
    final social = NdkNostrSocial(
      ndk: ndk,
      relays: [RelayUrl.parse('wss://relay.example')],
    );
    final target = ProfileId.parse(Nip19.encodePubKey(testCreatorPublicKey));
    expect(await social.toggleBlock(target), isTrue);

    final pending = social.toggleBlock(target);
    await Future<void>.delayed(Duration.zero);
    account = testFanPublicKey;
    secondRead.complete();

    expect(await pending, isFalse);
    verify(() => broadcast.broadcast(
          nostrEvent: any(named: 'nostrEvent'),
          specificRelays: any(named: 'specificRelays'),
          customSigner: signer,
          saveToCache: false,
        )).called(2);
  });
}
