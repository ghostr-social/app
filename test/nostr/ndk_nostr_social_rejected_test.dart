import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:mocktail/mocktail.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/social_broadcast_harness.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('rejects a social mutation when no relay accepts the event', () async {
    final harness = SocialBroadcastHarness();
    harness.port.failure = const AppFailure('No Nostr relay accepted.');
    final social = harness.build();

    await expectLater(
      social.toggleFollow(ProfileId.parse(testViewerNpub)),
      throwsA(isA<AppFailure>().having(
        (failure) => failure.message,
        'message',
        contains('No Nostr relay'),
      )),
    );
    expect(harness.port.broadcasts, isEmpty);
    verifyNever(() => harness.cache.saveEvent(any()));
    verifyNever(() => harness.cache.saveContactList(any()));
  });
}
