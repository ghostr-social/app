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

  test('removes duplicate contacts and adds one replacement contact', () async {
    final harness = SocialBroadcastHarness();
    harness.events.events.add(socialEvent(
      identity: socialEventIdentity(1, ContactList.kKind, 10),
      tags: const [
        ['p', testFanPublicKey, 'first', 'one'],
        ['p', testCreatorPublicKey, 'creator-relay', 'friend'],
        ['p', testFanPublicKey, 'second', 'two'],
      ],
    ));
    final social = harness.build();
    final target = ProfileId.parse(Nip19.encodePubKey(testFanPublicKey));

    expect(await social.toggleFollow(target), isFalse);
    expect(await social.toggleFollow(target), isTrue);

    final events = harness.port.broadcasts.map(decodeSignedNostrEvent).toList();
    expect(events.first.tags, [
      ['p', testCreatorPublicKey, 'creator-relay', 'friend'],
    ]);
    expect(events.last.pTags, {testCreatorPublicKey, testFanPublicKey});
  });
}
