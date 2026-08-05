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

  test('mute mutation decrypts and preserves public and private list tags',
      () async {
    final harness = SocialBroadcastHarness();
    harness.events.events.add(socialEvent(
      identity: socialEventIdentity(1, Nip51List.kMute, 10),
      tags: const [
        ['p', testCreatorPublicKey],
        ['t', 'public-tag'],
        ['e', testEventId],
        ['a', '34550:author:public'],
      ],
      content: 'ciphertext',
    ));
    when(() => harness.signer.decryptNip44(
          ciphertext: 'ciphertext',
          senderPubKey: testViewerPublicKey,
        )).thenAnswer((_) async {
      return jsonEncode([
        ['p', testFanPublicKey],
        ['t', 'private-tag'],
        ['e', secondTestEventId],
        ['a', '34550:author:private'],
      ]);
    });
    final social = harness.build();
    final target = ProfileId.parse(Nip19.encodePubKey(testFanPublicKey));

    expect(await social.toggleBlock(target), isFalse);

    final event = decodeSignedNostrEvent(harness.port.broadcasts.single);
    expect(event.tags, [
      ['p', testCreatorPublicKey],
      ['t', 'public-tag'],
      ['e', testEventId],
      ['a', '34550:author:public'],
    ]);
    final encrypted = verify(() => harness.signer.encryptNip44(
          plaintext: captureAny(named: 'plaintext'),
          recipientPubKey: testViewerPublicKey,
        )).captured.single as String;
    expect(jsonDecode(encrypted), [
      ['t', 'private-tag'],
      ['e', secondTestEventId],
      ['a', '34550:author:private'],
    ]);
  });
}
