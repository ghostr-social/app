import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/scripted_nostr_event_client.dart';
import '../support/social_broadcast_harness.dart';
import '../support/social_event_fixtures.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('newest valid event wins when a client returns multiple rows', () async {
    final client = ScriptedNostrEventClient((_) {
      return [
        _contacts(1, 10, testFanPublicKey),
        _contacts(2, 20, testCreatorPublicKey),
      ];
    });
    final social = SocialBroadcastHarness(events: client).build();

    final followed = await social.loadFollowedProfiles();

    expect(followed, {Nip19.encodePubKey(testCreatorPublicKey)});
  });
}

NostrEventRecord _contacts(int sequence, int createdAt, String target) {
  return socialEvent(
    identity: socialEventIdentity(sequence, ContactList.kKind, createdAt),
    tags: [
      ['p', target],
    ],
  );
}
