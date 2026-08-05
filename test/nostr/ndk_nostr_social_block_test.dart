import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/platform/nostr/signed_nostr_event_json.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/social_broadcast_harness.dart';
import '../support/social_event_fixtures.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('removes and adds a private NIP-51 mute-list entry', () async {
    final harness = SocialBroadcastHarness();
    harness.events.events.add(socialEvent(
      identity: socialEventIdentity(1, Nip51List.kMute, 10),
      content: 'ciphertext',
    ));
    when(() => harness.signer.decryptNip44(
          ciphertext: 'ciphertext',
          senderPubKey: testViewerPublicKey,
        )).thenAnswer((_) async {
      return jsonEncode([
        ['p', testFanPublicKey],
      ]);
    });
    final social = harness.build();
    final target = ProfileId.parse(Nip19.encodePubKey(testFanPublicKey));

    expect(await social.toggleBlock(target), isFalse);
    expect(await social.toggleBlock(target), isTrue);

    final events = harness.port.broadcasts.map(decodeSignedNostrEvent).toList();
    expect(events.first.content, isEmpty);
    expect(events.last.content, 'encrypted');
    verify(() => harness.signer.encryptNip44(
          plaintext: any(named: 'plaintext'),
          recipientPubKey: testViewerPublicKey,
        )).called(1);
  });
}
