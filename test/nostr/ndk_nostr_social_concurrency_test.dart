import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/platform/nostr/signed_nostr_event_json.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/scripted_nostr_event_client.dart';
import '../support/social_broadcast_harness.dart';
import '../support/social_event_fixtures.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('serializes follows and issues monotonic replacement events', () async {
    final started = Completer<void>();
    final release = Completer<void>();
    var reads = 0;
    final client = ScriptedNostrEventClient((_) async {
      reads += 1;
      started.complete();
      await release.future;
      return [_contacts()];
    });
    final harness = SocialBroadcastHarness(events: client);
    final social = harness.build(
      clock: () => DateTime.fromMillisecondsSinceEpoch(100000),
    );

    final first = social.toggleFollow(_profile(testFanPublicKey));
    await started.future;
    final second = social.toggleFollow(_profile(testAuthorPublicKey));
    await Future<void>.delayed(Duration.zero);
    expect(reads, 1);
    harness.activePublicKey = testCreatorPublicKey;
    release.complete();
    await Future.wait(<Future<bool>>[first, second]);

    final events = harness.port.broadcasts.map(decodeSignedNostrEvent).toList();
    expect(events[1].createdAt, greaterThan(events[0].createdAt));
    expect(events[1].pTags, {testFanPublicKey, testAuthorPublicKey});
    expect(reads, 1);
    final cached = verify(() => harness.cache.saveEvent(captureAny())).captured;
    expect(cached.cast<Nip01Event>().map((event) => event.sig),
        everyElement('sig'));
    verify(() => harness.cache.saveContactList(any())).called(2);
  });
}

NostrEventRecord _contacts() {
  return socialEvent(
    identity: socialEventIdentity(1, ContactList.kKind, 10),
  );
}

ProfileId _profile(String publicKey) {
  return ProfileId.parse(Nip19.encodePubKey(publicKey));
}
