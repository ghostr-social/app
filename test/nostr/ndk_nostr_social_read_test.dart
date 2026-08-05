import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/social_broadcast_harness.dart';
import '../support/social_event_fixtures.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('loads NIP-02 follows and private NIP-51 mutes as npubs', () async {
    final harness = SocialBroadcastHarness();
    harness.events.events.addAll([
      socialEvent(
        identity: socialEventIdentity(1, ContactList.kKind, 10),
        tags: const [
          ['p', testCreatorPublicKey],
        ],
      ),
      socialEvent(
        identity: socialEventIdentity(2, Nip51List.kMute, 10),
        content: 'ciphertext',
      ),
    ]);
    when(() => harness.signer.decryptNip44(
          ciphertext: 'ciphertext',
          senderPubKey: testViewerPublicKey,
        )).thenAnswer((_) async {
      return jsonEncode([
        ['p', testFanPublicKey],
      ]);
    });
    final social = harness.build();

    final blocked = await social.loadBlockedProfiles();
    final followed = await social.loadFollowedProfiles();

    expect(blocked, {Nip19.encodePubKey(testFanPublicKey)});
    expect(followed, {Nip19.encodePubKey(testCreatorPublicKey)});
  });
}
