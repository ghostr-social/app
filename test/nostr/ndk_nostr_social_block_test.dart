import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_social.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('removes and adds a private NIP-51 mute-list entry', () async {
    const publicKey =
        '7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e';
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final lists = MockLists();
    final broadcast = MockBroadcast();
    final response = MockNdkBroadcastResponse();
    final signer = MockEventSigner();
    final populated = _muteList(publicKey, includeTarget: true);
    final empty = _muteList(publicKey, includeTarget: false);
    var readCount = 0;
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.lists).thenReturn(lists);
    when(() => ndk.broadcast).thenReturn(broadcast);
    when(accounts.getPublicKey).thenReturn('viewer');
    when(accounts.getLoggedAccount).thenReturn(Account(
      type: AccountType.privateKey,
      pubkey: 'viewer',
      signer: signer,
    ));
    when(signer.getPublicKey).thenReturn('viewer');
    when(signer.canSign).thenReturn(true);
    stubEventSigner(signer, 'viewer');
    when(() => signer.encryptNip44(
          plaintext: any(named: 'plaintext'),
          recipientPubKey: 'viewer',
        )).thenAnswer((_) async => 'encrypted');
    when(() => response.broadcastDoneFuture)
        .thenAnswer((_) async => successfulRelayBroadcast());
    when(() => lists.getSingleNip51List(Nip51List.kMute)).thenAnswer(
      (_) async => readCount++ == 0 ? populated : empty,
    );
    when(() => lists.getSingleNip51List(
          Nip51List.kMute,
          forceRefresh: true,
        )).thenAnswer((_) async {
      return _muteList(publicKey, includeTarget: false)..createdAt = 5;
    });
    when(() => broadcast.broadcast(
          nostrEvent: any(named: 'nostrEvent'),
          specificRelays: any(named: 'specificRelays'),
          customSigner: signer,
          saveToCache: false,
        )).thenReturn(response);
    final runtime = NdkNostrSocial(
      ndk: ndk,
      relays: [RelayUrl.parse('wss://relay.example')],
    );
    final npub = ProfileId.parse(Nip19.encodePubKey(publicKey));

    expect(await runtime.toggleBlock(npub), isFalse);
    expect(await runtime.toggleBlock(npub), isTrue);

    verify(() => broadcast.broadcast(
          nostrEvent: any(named: 'nostrEvent'),
          specificRelays: ['wss://relay.example'],
          customSigner: signer,
          saveToCache: false,
        )).called(2);
    verify(() => signer.encryptNip44(
          plaintext: any(named: 'plaintext'),
          recipientPubKey: 'viewer',
        )).called(1);
  });
}

Nip51List _muteList(String target, {required bool includeTarget}) {
  return Nip51List(
    pubKey: target,
    kind: Nip51List.kMute,
    createdAt: 10,
    elements: includeTarget
        ? [
            Nip51ListElement(
              tag: Nip51List.kPubkey,
              value: target,
              private: true,
            ),
          ]
        : [],
  );
}
