import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_social.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('rejects when a refreshed mute list crosses an account switch',
      () async {
    var account = testViewerPublicKey;
    final refresh = Completer<Nip51List?>();
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final lists = MockLists();
    final signer = MockEventSigner();
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.lists).thenReturn(lists);
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
    when(() => lists.getSingleNip51List(Nip51List.kMute))
        .thenAnswer((_) async => null);
    when(() => lists.getSingleNip51List(
          Nip51List.kMute,
          forceRefresh: true,
        )).thenAnswer((_) => refresh.future);
    final social = NdkNostrSocial(ndk: ndk, relays: const []);

    final pending = social.toggleBlock(ProfileId.parse(testViewerNpub));
    await Future<void>.delayed(Duration.zero);
    account = testAuthorPublicKey;
    refresh.complete(_muteList(testAuthorPublicKey));

    await expectLater(pending, throwsA(isA<AppFailure>()));
    verifyNever(() => signer.sign(any()));
    verifyNever(() => ndk.config);
  });
}

Nip51List _muteList(String publicKey) {
  return Nip51List(
    pubKey: publicKey,
    kind: Nip51List.kMute,
    createdAt: 10,
    elements: <Nip51ListElement>[
      Nip51ListElement(
        tag: Nip51List.kPubkey,
        value: testFanPublicKey,
        private: true,
      ),
    ],
  );
}
