import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/scripted_nostr_event_client.dart';
import '../support/social_broadcast_harness.dart';
import '../support/social_event_fixtures.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('rejects when a mute event query crosses an account switch', () async {
    final started = Completer<void>();
    final response = Completer<List<NostrEventRecord>>();
    final client = ScriptedNostrEventClient((_) {
      started.complete();
      return response.future;
    });
    final harness = SocialBroadcastHarness(events: client);
    final social = harness.build();

    final pending = social.toggleBlock(ProfileId.parse(testViewerNpub));
    await started.future;
    harness.activePublicKey = testAuthorPublicKey;
    response.complete([_muteFromNextAccount()]);

    await expectLater(pending, throwsA(isA<AppFailure>()));
    verifyNever(() => harness.signer.sign(any()));
    verifyNever(() => harness.ndk.config);
  });
}

NostrEventRecord _muteFromNextAccount() {
  return socialEvent(
    identity: socialEventIdentity(
      1,
      Nip51List.kMute,
      10,
      testAuthorPublicKey,
    ),
    tags: const [
      ['p', testFanPublicKey],
    ],
  );
}
