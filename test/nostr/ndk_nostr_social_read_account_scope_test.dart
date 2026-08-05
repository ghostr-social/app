import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/social_broadcast_harness.dart';
import '../support/social_event_fixtures.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('a pinned follow read never switches to the next account', () async {
    final harness = SocialBroadcastHarness();
    harness.events.events.add(socialEvent(
      identity: socialEventIdentity(1, ContactList.kKind, 10),
      tags: const [
        ['p', testFanPublicKey],
      ],
    ));
    final scoped = harness.build().snapshotForActiveAccount();
    harness.activePublicKey = testAuthorPublicKey;

    final followed = await scoped.loadFollowedProfiles();

    expect(followed, {Nip19.encodePubKey(testFanPublicKey)});
    expect(harness.events.queries.single.authors.single.value,
        testViewerPublicKey);
  });

  test('a pinned private mute read rejects an account switch', () async {
    final harness = SocialBroadcastHarness();
    final scoped = harness.build().snapshotForActiveAccount();
    harness.activePublicKey = testAuthorPublicKey;

    await expectLater(
      scoped.loadBlockedProfiles(),
      throwsA(isA<AppFailure>()),
    );
    expect(harness.events.queries, isEmpty);
  });
}
