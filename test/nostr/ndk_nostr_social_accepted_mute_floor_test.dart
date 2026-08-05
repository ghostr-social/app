import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/scripted_nostr_event_client.dart';
import '../support/social_broadcast_harness.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('accepted mute floor survives an account switch during reread',
      () async {
    final secondRead = Completer<List<NostrEventRecord>>();
    var reads = 0;
    final client = ScriptedNostrEventClient((_) {
      reads += 1;
      if (reads == 1) return <NostrEventRecord>[];
      return secondRead.future;
    });
    final harness = SocialBroadcastHarness(events: client);
    final social = harness.build();
    final target = ProfileId.parse(Nip19.encodePubKey(testCreatorPublicKey));
    expect(await social.toggleBlock(target), isTrue);

    final pending = social.toggleBlock(target);
    await Future<void>.delayed(Duration.zero);
    harness.activePublicKey = testFanPublicKey;
    secondRead.complete(<NostrEventRecord>[]);

    expect(await pending, isFalse);
    expect(harness.port.broadcasts, hasLength(2));
  });
}
