import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_social.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('keeps follow mutation bound to the initiating signer', () async {
    const target =
        '7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e';
    var account = 'account-a';
    final barrier = Completer<ContactList?>();
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final follows = MockFollows();
    final broadcast = MockBroadcast();
    final response = MockNdkBroadcastResponse();
    final signer = MockEventSigner();
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.follows).thenReturn(follows);
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
    when(() => follows.getContactList(
          'account-a',
          forceRefresh: true,
        )).thenAnswer((_) => barrier.future);
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

    final pending = social.toggleFollow(
      ProfileId.parse(Nip19.encodePubKey(target)),
    );
    await Future<void>.delayed(Duration.zero);
    account = 'account-b';
    barrier.complete(ContactList(pubKey: 'account-a', contacts: <String>[]));

    await expectLater(pending, completion(isTrue));
    verifyNever(() => follows.broadcastAddContact(
          any(),
          customRelays: any(named: 'customRelays'),
        ));
    verify(() => broadcast.broadcast(
          nostrEvent: any(named: 'nostrEvent'),
          specificRelays: ['wss://relay.example'],
          customSigner: signer,
          saveToCache: false,
        )).called(1);
  });
}
