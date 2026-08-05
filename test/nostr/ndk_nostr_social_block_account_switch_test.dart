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

void main() {
  setUpAll(registerNdkFallbackValues);

  test('rejects a first mute mutation after its account query switches',
      () async {
    final started = Completer<void>();
    final response = Completer<List<NostrEventRecord>>();
    final client = ScriptedNostrEventClient((_) {
      started.complete();
      return response.future;
    });
    final harness = SocialBroadcastHarness(events: client);
    final social = harness.build();

    final pending = social.toggleBlock(
      ProfileId.parse(Nip19.encodePubKey(testFanPublicKey)),
    );
    await started.future;
    harness.activePublicKey = testAuthorPublicKey;
    response.complete(<NostrEventRecord>[]);

    await expectLater(pending, throwsA(isA<AppFailure>()));
    verifyNever(() => harness.signer.sign(any()));
    verifyNever(() => harness.ndk.config);
    expect(harness.port.broadcasts, isEmpty);
  });
}
