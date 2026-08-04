import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/social/domain/signed_event_broadcast_port.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_social.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import 'ndk_mocks.dart';
import 'nostr_test_values.dart';

/// Captures the canonical JSON handed to the transport by a mutation.
final class RecordingBroadcastPort implements SignedEventBroadcastPort {
  final broadcasts = <String>[];

  @override
  Future<void> broadcast(String signedEventJson) async {
    broadcasts.add(signedEventJson);
  }
}

/// A signed-in [NdkNostrSocial] whose writes end at a recording port.
final class SocialBroadcastHarness {
  final ndk = MockNdk();
  final accounts = MockAccounts();
  final follows = MockFollows();
  final lists = MockLists();
  final config = MockNdkConfig();
  final cache = MockCacheManager();
  final signer = MockEventSigner();
  final port = RecordingBroadcastPort();

  NdkNostrSocial build() {
    _stubNdk();
    _stubAccount();
    when(() => cache.saveEvent(any())).thenAnswer((_) async {});
    when(() => cache.saveContactList(any())).thenAnswer((_) async {});
    return NdkNostrSocial(
      ndk: ndk,
      relays: [RelayUrl.parse('wss://relay.example')],
      broadcast: port,
    );
  }

  void _stubNdk() {
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.follows).thenReturn(follows);
    when(() => ndk.lists).thenReturn(lists);
    when(() => ndk.config).thenReturn(config);
    when(() => config.cache).thenReturn(cache);
  }

  void _stubAccount() {
    when(accounts.getPublicKey).thenReturn(testViewerPublicKey);
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
  }
}
