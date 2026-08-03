import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_social.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('serializes follows and issues monotonic replacement events', () async {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final follows = MockFollows();
    final broadcast = MockBroadcast();
    final signer = MockEventSigner();
    final config = MockNdkConfig();
    final cache = MockCacheManager();
    final started = Completer<void>();
    final release = Completer<void>();
    final events = <Nip01Event>[];
    var reads = 0;
    var activeAccount = testViewerPublicKey;
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.follows).thenReturn(follows);
    when(() => ndk.broadcast).thenReturn(broadcast);
    when(() => ndk.config).thenReturn(config);
    when(() => config.cache).thenReturn(cache);
    when(accounts.getPublicKey).thenAnswer((_) => activeAccount);
    when(() => accounts.getLoggedAccount()).thenReturn(Account(
      type: AccountType.privateKey,
      pubkey: testViewerPublicKey,
      signer: signer,
    ));
    stubEventSigner(signer, testViewerPublicKey);
    when(() => cache.saveEvent(any())).thenAnswer((_) async {});
    when(() => cache.saveContactList(any())).thenAnswer((_) async {});
    when(() => follows.getContactList(
          testViewerPublicKey,
          forceRefresh: true,
        )).thenAnswer((_) async {
      reads += 1;
      if (reads == 1) {
        started.complete();
        await release.future;
      }
      return ContactList(pubKey: testViewerPublicKey, contacts: <String>[])
        ..createdAt = 10;
    });
    when(() => broadcast.broadcast(
          nostrEvent: any(named: 'nostrEvent'),
          specificRelays: any(named: 'specificRelays'),
          customSigner: signer,
          saveToCache: false,
        )).thenAnswer((call) {
      final event = call.namedArguments[#nostrEvent] as Nip01Event;
      events.add(event);
      return NdkBroadcastResponse(
        publishEvent: event,
        broadcastDoneStream: Stream.value(successfulRelayBroadcast()),
      );
    });
    final social = NdkNostrSocial(
      ndk: ndk,
      relays: const [],
      clock: () => DateTime.fromMillisecondsSinceEpoch(100000),
    );

    final first = social.toggleFollow(_profile(testFanPublicKey));
    await started.future;
    final second = social.toggleFollow(_profile(testAuthorPublicKey));
    await Future<void>.delayed(Duration.zero);
    expect(reads, 1);
    activeAccount = testFanPublicKey;
    release.complete();
    await Future.wait(<Future<bool>>[first, second]);

    expect(events[1].createdAt, greaterThan(events[0].createdAt));
    expect(events[1].pTags, {testFanPublicKey, testAuthorPublicKey});
    expect(reads, 1);
    final cached = verify(() => cache.saveEvent(captureAny())).captured;
    expect(cached.cast<Nip01Event>().map((event) => event.sig),
        everyElement('sig'));
    verify(() => cache.saveContactList(any())).called(2);
  });
}

ProfileId _profile(String publicKey) {
  return ProfileId.parse(Nip19.encodePubKey(publicKey));
}
