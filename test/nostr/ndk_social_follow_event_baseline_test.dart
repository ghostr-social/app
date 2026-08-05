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

  test('follow mutation preserves auxiliary tags from the queried event',
      () async {
    final harness = SocialBroadcastHarness();
    harness.events.events.add(socialEvent(
      identity: socialEventIdentity(1, ContactList.kKind, 10),
      tags: const [
        ['p', testFanPublicKey, 'wss://fan.example', 'fan'],
        ['p', testCreatorPublicKey, 'wss://creator.example', 'creator'],
        ['t', 'dance'],
        ['a', '34550:author:community'],
        ['e', testEventId],
      ],
    ));
    final social = harness.build();
    final target = ProfileId.parse(Nip19.encodePubKey(testFanPublicKey));

    expect(await social.toggleFollow(target), isFalse);

    final event = decodeSignedNostrEvent(harness.port.broadcasts.single);
    expect(event.tags, [
      ['p', testCreatorPublicKey, 'wss://creator.example', 'creator'],
      ['t', 'dance'],
      ['a', '34550:author:community'],
      ['e', testEventId],
    ]);
  });
}
