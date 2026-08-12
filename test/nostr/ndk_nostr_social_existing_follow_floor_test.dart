import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/platform/nostr/signed_nostr_event_json.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/social_broadcast_harness.dart';
import '../support/social_event_fixtures.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test(
    'an observed existing follow protects contacts from stale reads',
    () async {
      final harness = SocialBroadcastHarness();
      harness.events.events.add(
        socialEvent(
          identity: socialEventIdentity(1, ContactList.kKind, 10),
          tags: const [
            ['p', testCreatorPublicKey, '', ''],
            ['p', testAuthorPublicKey, '', ''],
          ],
        ),
      );
      final social = harness.build();

      await social.follow(_profile(testCreatorPublicKey));
      harness.events.events.clear();
      await social.follow(_profile(testFanPublicKey));

      final event = decodeSignedNostrEvent(harness.port.broadcasts.single);
      expect(event.tags, [
        ['p', testCreatorPublicKey, '', ''],
        ['p', testAuthorPublicKey, '', ''],
        ['p', testFanPublicKey, '', ''],
      ]);
    },
  );
}

ProfileId _profile(String publicKey) {
  return ProfileId.parse(Nip19.encodePubKey(publicKey));
}
