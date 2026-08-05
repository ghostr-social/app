import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/social_broadcast_harness.dart';
import '../support/social_event_fixtures.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('stale follow events cannot undo an accepted replacement', () async {
    final harness = SocialBroadcastHarness();
    harness.events.events.add(_contacts(1, 10));
    final social = harness.build(
      clock: () => DateTime.fromMillisecondsSinceEpoch(20000),
    );
    final target = ProfileId.parse(Nip19.encodePubKey(testFanPublicKey));

    expect(await social.toggleFollow(target), isTrue);
    expect(await social.loadFollowedProfiles(), {target});
    harness.events.events
      ..clear()
      ..add(_contacts(2, 30));
    expect(await social.loadFollowedProfiles(), isEmpty);
    harness.events.events
      ..clear()
      ..add(_contacts(3, 10));
    expect(await social.loadFollowedProfiles(), isEmpty);
  });
}

NostrEventRecord _contacts(int sequence, int createdAt) {
  return socialEvent(
    identity: socialEventIdentity(sequence, ContactList.kKind, createdAt),
  );
}
