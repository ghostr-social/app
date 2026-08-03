import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_social.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('removes and adds a contact through NIP-02', () async {
    const publicKey =
        '7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e';
    const other =
        '8e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e';
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final follows = MockFollows();
    final broadcast = MockBroadcast();
    final response = MockNdkBroadcastResponse();
    final signer = MockEventSigner();
    final populated = ContactList(
      pubKey: 'viewer',
      contacts: [publicKey, other, publicKey],
    )
      ..contactRelays = ['first', 'other-relay', 'second']
      ..petnames = ['one', 'friend', 'two'];
    var readCount = 0;
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.follows).thenReturn(follows);
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
    when(() => response.broadcastDoneFuture)
        .thenAnswer((_) async => successfulRelayBroadcast());
    when(() => follows.getContactList('viewer', forceRefresh: true)).thenAnswer(
      (_) async => ContactList(
        pubKey: 'viewer',
        contacts: readCount++ == 0 ? populated.contacts : [],
      )
        ..contactRelays = readCount == 1 ? populated.contactRelays : []
        ..petnames = readCount == 1 ? populated.petnames : [],
    );
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

    expect(await runtime.toggleFollow(npub), isFalse);
    expect(await runtime.toggleFollow(npub), isTrue);

    final events = verify(() => broadcast.broadcast(
          nostrEvent: captureAny(named: 'nostrEvent'),
          specificRelays: ['wss://relay.example'],
          customSigner: signer,
          saveToCache: false,
        )).captured.cast<Nip01Event>();
    expect(events, hasLength(2));
    expect(events.first.tags.where((tag) => tag.first == 'p'), [
      ['p', other, 'other-relay', 'friend'],
    ]);
    expect(populated.contacts, [publicKey, other, publicKey]);
    expect(
      events.last.tags,
      contains(predicate<List<String>>(
        (tag) => tag.length >= 2 && tag[0] == 'p' && tag[1] == publicKey,
      )),
    );
  });
}
