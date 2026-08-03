import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_social.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('rejects a first mute mutation after its account read switches',
      () async {
    const target =
        '7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e';
    var account = 'account-a';
    final barrier = Completer<Nip51List?>();
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
      pubkey: 'account-a',
      signer: signer,
    ));
    when(signer.getPublicKey).thenReturn('account-a');
    when(signer.canSign).thenReturn(true);
    stubEventSigner(signer, 'account-a');
    when(() => response.broadcastDoneFuture)
        .thenAnswer((_) async => successfulRelayBroadcast());
    when(() => signer.encryptNip44(
          plaintext: any(named: 'plaintext'),
          recipientPubKey: 'account-a',
        )).thenAnswer((_) async => 'encrypted');
    when(() => lists.getSingleNip51List(Nip51List.kMute))
        .thenAnswer((_) => barrier.future);
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

    final pending = social.toggleBlock(
      ProfileId.parse(Nip19.encodePubKey(target)),
    );
    await Future<void>.delayed(Duration.zero);
    account = 'account-b';
    barrier.complete(Nip51List(
      pubKey: 'account-b',
      kind: Nip51List.kMute,
      createdAt: 10,
      elements: <Nip51ListElement>[
        Nip51ListElement(tag: Nip51List.kPubkey, value: target, private: true),
      ],
    ));

    await expectLater(pending, throwsA(isA<AppFailure>()));
    verifyNever(() => signer.sign(any()));
    verifyNever(() => ndk.config);
    verifyNever(() => lists.addElementToList(
          kind: any(named: 'kind'),
          tag: any(named: 'tag'),
          value: any(named: 'value'),
          broadcastRelays: any(named: 'broadcastRelays'),
          private: any(named: 'private'),
        ));
    verifyNever(() => broadcast.broadcast(
          nostrEvent: any(named: 'nostrEvent'),
          specificRelays: any(named: 'specificRelays'),
          customSigner: any(named: 'customSigner'),
          saveToCache: any(named: 'saveToCache'),
        ));
  });
}
