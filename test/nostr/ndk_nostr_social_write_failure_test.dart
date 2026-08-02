import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_social.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:mocktail/mocktail.dart';

import '../support/ndk_mocks.dart';

void main() {
  test('rejects social writes without a Nostr profile identifier', () async {
    final ndk = MockNdk();
    final runtime = NdkNostrSocial(ndk: ndk, relays: const []);

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
    verifyNever(() => ndk.lists);
    verifyNever(() => ndk.follows);
  });
}
