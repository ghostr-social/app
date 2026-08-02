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
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final follows = MockFollows();
    var readCount = 0;
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.follows).thenReturn(follows);
    when(accounts.getPublicKey).thenReturn('viewer');
    when(() => follows.getContactList('viewer')).thenAnswer(
      (_) async => ContactList(
        pubKey: 'viewer',
        contacts: readCount++ == 0 ? [publicKey] : [],
      ),
    );
    when(
      () => follows.broadcastRemoveContact(
        publicKey,
        customRelays: any(named: 'customRelays'),
      ),
    ).thenAnswer(
      (_) async => ContactList(pubKey: 'viewer', contacts: const []),
    );
    when(
      () => follows.broadcastAddContact(
        publicKey,
        customRelays: any(named: 'customRelays'),
      ),
    ).thenAnswer(
      (_) async => ContactList(pubKey: 'viewer', contacts: [publicKey]),
    );
    final runtime = NdkNostrSocial(
      ndk: ndk,
      relays: [RelayUrl.parse('wss://relay.example')],
    );
    final npub = ProfileId.parse(Nip19.encodePubKey(publicKey));

    expect(await runtime.toggleFollow(npub), isFalse);
    expect(await runtime.toggleFollow(npub), isTrue);

    verify(
      () => follows.broadcastAddContact(
        publicKey,
        customRelays: ['wss://relay.example'],
      ),
    ).called(1);
  });
}
