import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_social.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('removes and adds a private NIP-51 mute-list entry', () async {
    const publicKey =
        '7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e';
    final ndk = MockNdk();
    final lists = MockLists();
    final populated = _muteList(publicKey, includeTarget: true);
    final empty = _muteList(publicKey, includeTarget: false);
    var readCount = 0;
    when(() => ndk.lists).thenReturn(lists);
    when(() => lists.getSingleNip51List(Nip51List.kMute)).thenAnswer(
      (_) async => readCount++ == 0 ? populated : empty,
    );
    when(
      () => lists.removeElementFromList(
        kind: any(named: 'kind'),
        tag: any(named: 'tag'),
        value: any(named: 'value'),
        broadcastRelays: any(named: 'broadcastRelays'),
      ),
    ).thenAnswer((_) async => empty);
    when(
      () => lists.addElementToList(
        kind: any(named: 'kind'),
        tag: any(named: 'tag'),
        value: any(named: 'value'),
        broadcastRelays: any(named: 'broadcastRelays'),
        private: any(named: 'private'),
      ),
    ).thenAnswer((_) async => populated);
    final runtime = NdkNostrSocial(
      ndk: ndk,
      relays: [RelayUrl.parse('wss://relay.example')],
    );
    final npub = ProfileId.parse(Nip19.encodePubKey(publicKey));

    expect(await runtime.toggleBlock(npub), isFalse);
    expect(await runtime.toggleBlock(npub), isTrue);

    verify(
      () => lists.addElementToList(
        kind: Nip51List.kMute,
        tag: Nip51List.kPubkey,
        value: publicKey,
        broadcastRelays: ['wss://relay.example'],
        private: true,
      ),
    ).called(1);
  });
}

Nip51List _muteList(String target, {required bool includeTarget}) {
  return Nip51List(
    pubKey: target,
    kind: Nip51List.kMute,
    createdAt: 10,
    elements: includeTarget
        ? [
            Nip51ListElement(
              tag: Nip51List.kPubkey,
              value: target,
              private: true,
            ),
          ]
        : [],
  );
}
