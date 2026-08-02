import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_social.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';

void main() {
  test('loads NIP-02 follows and NIP-51 muted profiles as npubs', () async {
    const publicKey =
        '7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e';
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final lists = MockLists();
    final follows = MockFollows();
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.lists).thenReturn(lists);
    when(() => ndk.follows).thenReturn(follows);
    when(accounts.getPublicKey).thenReturn(publicKey);
    when(() => lists.getSingleNip51List(Nip51List.kMute)).thenAnswer(
      (_) async => Nip51List(
        pubKey: publicKey,
        kind: Nip51List.kMute,
        createdAt: 10,
        elements: [
          Nip51ListElement(
            tag: Nip51List.kPubkey,
            value: publicKey,
            private: true,
          ),
        ],
      ),
    );
    when(() => follows.getContactList(publicKey)).thenAnswer(
      (_) async => ContactList(pubKey: publicKey, contacts: [publicKey]),
    );
    final runtime = NdkNostrSocial(ndk: ndk, relays: const []);

    final blocked = await runtime.loadBlockedProfiles();
    final followed = await runtime.loadFollowedProfiles();

    expect(blocked, {Nip19.encodePubKey(publicKey)});
    expect(followed, {Nip19.encodePubKey(publicKey)});
  });
}
