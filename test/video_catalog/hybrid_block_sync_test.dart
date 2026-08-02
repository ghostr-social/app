import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fakes.dart';
import '../support/hybrid_repository_harness.dart';

void main() {
  test('publishes a block and mirrors it into the catalog cache', () async {
    final social = FakeNostrSocialPort();
    final harness = await buildHybridRepositoryHarness(
      FakeRemoteVideoSource([]),
      ports: HybridHarnessPorts(social: social),
    );

    final creator = ProfileId.parse('creator');
    final blocked = await harness.profile.toggleBlock(creator);

    expect(blocked, isTrue);
    expect(social.blockedProfiles, {creator});
    expect(await harness.localStore.loadBlockedProfiles(), {creator});
  });
}
