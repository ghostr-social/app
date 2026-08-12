import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/domain/follow_outcome.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/social_broadcast_harness.dart';
import '../support/social_event_fixtures.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('following an existing contact never broadcasts an unfollow', () async {
    final harness = SocialBroadcastHarness();
    harness.events.events.add(
      socialEvent(
        identity: socialEventIdentity(1, ContactList.kKind, 10),
        tags: const [
          ['p', testCreatorPublicKey, '', ''],
        ],
      ),
    );
    final social = harness.build();
    final target = ProfileId.parse(Nip19.encodePubKey(testCreatorPublicKey));

    final outcome = await social.follow(target);

    expect(outcome, FollowOutcome.alreadyFollowing);
    expect(harness.port.broadcasts, isEmpty);
  });
}
