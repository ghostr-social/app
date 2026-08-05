import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/ndk_mocks.dart';
import '../support/social_broadcast_harness.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('rejects social writes without a Nostr profile identifier', () async {
    final harness = SocialBroadcastHarness();
    final runtime = harness.build();

    await expectLater(
      runtime.toggleBlock(ProfileId.parse('local-creator')),
      throwsA(isA<AppFailure>().having(
        (failure) => failure.message,
        'message',
        'This creator has no Nostr public key.',
      )),
    );
    await expectLater(
      runtime.toggleFollow(ProfileId.parse('local-creator')),
      throwsA(isA<AppFailure>().having(
        (failure) => failure.message,
        'message',
        'This creator has no Nostr public key.',
      )),
    );
    expect(harness.events.queries, isEmpty);
    expect(harness.port.broadcasts, isEmpty);
  });
}
