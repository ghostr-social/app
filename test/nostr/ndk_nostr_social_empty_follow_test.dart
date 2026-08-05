import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/platform/nostr/signed_nostr_event_json.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/social_broadcast_harness.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('follows from an account without a prior contact list', () async {
    final harness = SocialBroadcastHarness();
    final social = harness.build();

    final followed = await social.toggleFollow(
      ProfileId.parse(Nip19.encodePubKey(testCreatorPublicKey)),
    );

    expect(followed, isTrue);
    final event = decodeSignedNostrEvent(harness.port.broadcasts.single);
    expect(event.tags.single, ['p', testCreatorPublicKey, '', '']);
  });
}
