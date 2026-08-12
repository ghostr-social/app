import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/social_broadcast_harness.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('a mute publish keeps known blocks the relays failed to return',
      () async {
    final harness = SocialBroadcastHarness();
    final social = harness.build();
    final target = ProfileId.parse(Nip19.encodePubKey(testFanPublicKey));
    final known = ProfileId.parse(Nip19.encodePubKey(testCreatorPublicKey));

    expect(
      await social.toggleBlock(target, knownBlocked: {known}),
      isTrue,
    );

    expect(harness.port.broadcasts, hasLength(1));
    final plaintext = verify(() => harness.signer.encryptNip44(
          plaintext: captureAny(named: 'plaintext'),
          recipientPubKey: testViewerPublicKey,
        )).captured.single as String;
    expect(plaintext, contains(testCreatorPublicKey));
    expect(plaintext, contains(testFanPublicKey));
  });
}
