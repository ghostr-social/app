import 'package:ghostr/core/time/clock.dart';
import 'package:ghostr/features/social/domain/signed_event_broadcast_port.dart';
import 'package:ghostr/features/social/domain/signed_nostr_event_json.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_social.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import 'fake_nostr_event_client.dart';
import 'ndk_mocks.dart';
import 'nostr_test_values.dart';

/// Captures the canonical JSON handed to the transport by a mutation.
final class RecordingBroadcastPort implements SignedEventBroadcastPort {
  final broadcasts = <SignedNostrEventJson>[];
  Object? failure;

  @override
  Future<void> broadcast(SignedNostrEventJson event) async {
    if (failure != null) throw failure!;
    broadcasts.add(event);
  }
}

/// A signed-in [NdkNostrSocial] whose writes end at a recording port.
final class SocialBroadcastHarness {
  SocialBroadcastHarness({FakeNostrEventClient? events})
      : events =
            events ?? FakeNostrEventClient(publicKeyHex: testViewerPublicKey);

  final ndk = MockNdk();
  final accounts = MockAccounts();
  final config = MockNdkConfig();
  final cache = MockCacheManager();
  final signer = MockEventSigner();
  final FakeNostrEventClient events;
  final port = RecordingBroadcastPort();
  String activePublicKey = testViewerPublicKey;

  NdkNostrSocial build({Clock clock = systemClock}) {
    _stubNdk();
    _stubAccount();
    when(() => cache.saveEvent(any())).thenAnswer((_) async {});
    when(() => cache.saveContactList(any())).thenAnswer((_) async {});
    return NdkNostrSocial(
      ndk: ndk,
      eventClient: events,
      broadcast: port,
      clock: clock,
    );
  }

  void _stubNdk() {
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.config).thenReturn(config);
    when(() => config.cache).thenReturn(cache);
  }

  void _stubAccount() {
    when(accounts.getPublicKey).thenAnswer((_) => activePublicKey);
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
