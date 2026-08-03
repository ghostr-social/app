import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_social.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('a pinned follow read never switches to the next account', () async {
    final setup = _setup();
    final follows = MockFollows();
    when(() => setup.ndk.follows).thenReturn(follows);
    when(() => follows.getContactList(testViewerPublicKey)).thenAnswer(
      (_) async => ContactList(
        pubKey: testViewerPublicKey,
        contacts: <String>[testFanPublicKey],
      ),
    );
    final scoped = setup.social.snapshotForActiveAccount();
    setup.active = testAuthorPublicKey;

    final followed = await scoped.loadFollowedProfiles();

    expect(followed, {Nip19.encodePubKey(testFanPublicKey)});
    verify(() => follows.getContactList(testViewerPublicKey)).called(1);
  });

  test('a pinned private mute read rejects an account switch', () async {
    final setup = _setup();
    final lists = MockLists();
    when(() => setup.ndk.lists).thenReturn(lists);
    final scoped = setup.social.snapshotForActiveAccount();
    setup.active = testAuthorPublicKey;

    await expectLater(
      scoped.loadBlockedProfiles(),
      throwsA(isA<AppFailure>()),
    );
    verifyNever(() => lists.getSingleNip51List(Nip51List.kMute));
  });
}

_SocialSetup _setup() {
  final ndk = MockNdk();
  final accounts = MockAccounts();
  final signer = MockEventSigner();
  final setup = _SocialSetup(ndk, accounts, signer);
  when(() => ndk.accounts).thenReturn(accounts);
  when(accounts.getPublicKey).thenAnswer((_) => setup.active);
  when(() => accounts.getLoggedAccount()).thenReturn(Account(
    type: AccountType.privateKey,
    pubkey: testViewerPublicKey,
    signer: signer,
  ));
  stubEventSigner(signer, testViewerPublicKey);
  return setup;
}

class _SocialSetup {
  _SocialSetup(this.ndk, this.accounts, this.signer)
      : social = NdkNostrSocial(ndk: ndk, relays: const []);

  final MockNdk ndk;
  final MockAccounts accounts;
  final MockEventSigner signer;
  final NdkNostrSocial social;
  String active = testViewerPublicKey;
}
