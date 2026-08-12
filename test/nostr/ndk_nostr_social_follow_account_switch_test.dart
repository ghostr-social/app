import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/platform/nostr/signed_nostr_event_json.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/scripted_nostr_event_client.dart';
import '../support/social_broadcast_harness.dart';
import '../support/social_event_fixtures.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('keeps follow state isolated to the initiating signer', () async {
    final started = Completer<void>();
    final response = Completer<List<NostrEventRecord>>();
    var queries = 0;
    final client = ScriptedNostrEventClient((_) {
      if (queries++ > 0) return <NostrEventRecord>[];
      started.complete();
      return response.future;
    });
    final harness = SocialBroadcastHarness(events: client);
    final social = harness.build();

    final pending = social.toggleFollow(
      ProfileId.parse(Nip19.encodePubKey(testFanPublicKey)),
    );
    await started.future;
    harness.activePublicKey = testAuthorPublicKey;
    response.complete([
      socialEvent(
        identity: socialEventIdentity(1, ContactList.kKind, 10),
        tags: const [
          ['p', testCreatorPublicKey, '', ''],
        ],
      ),
    ]);

    await expectLater(pending, completion(isTrue));
    final event = decodeSignedNostrEvent(harness.port.broadcasts.single);
    expect(event.pubKey, testViewerPublicKey);
    expect(event.pTags, {testCreatorPublicKey, testFanPublicKey});
    expect(await social.loadFollowedProfiles(), isEmpty);
  });
}
