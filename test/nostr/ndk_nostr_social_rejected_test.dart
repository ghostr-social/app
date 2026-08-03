import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_social.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('rejects a social mutation when no relay accepts the event', () async {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final follows = MockFollows();
    final broadcast = MockBroadcast();
    final response = MockNdkBroadcastResponse();
    final signer = MockEventSigner();
    final config = MockNdkConfig();
    final cache = MockCacheManager();
    final contacts = ContactList(
      pubKey: testViewerPublicKey,
      contacts: <String>[],
    );
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.follows).thenReturn(follows);
    when(() => ndk.broadcast).thenReturn(broadcast);
    when(() => ndk.config).thenReturn(config);
    when(() => config.cache).thenReturn(cache);
    when(accounts.getPublicKey).thenReturn(testViewerPublicKey);
    when(accounts.getLoggedAccount).thenReturn(Account(
      type: AccountType.privateKey,
      pubkey: testViewerPublicKey,
      signer: signer,
    ));
    when(signer.getPublicKey).thenReturn(testViewerPublicKey);
    when(signer.canSign).thenReturn(true);
    stubEventSigner(signer, testViewerPublicKey);
    when(() => follows.getContactList(
          testViewerPublicKey,
          forceRefresh: true,
        )).thenAnswer((_) async => contacts);
    when(() => response.broadcastDoneFuture).thenAnswer((_) async => []);
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

    await expectLater(
      social.toggleFollow(ProfileId.parse(testViewerNpub)),
      throwsA(isA<AppFailure>().having(
        (failure) => failure.message,
        'message',
        contains('No Nostr relay'),
      )),
    );
    expect(contacts.contacts, isEmpty);
    verifyNever(() => cache.saveEvent(any()));
    verifyNever(() => cache.saveContactList(any()));
  });
}
