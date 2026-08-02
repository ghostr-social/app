import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fakes.dart';
import '../support/hybrid_repository_harness.dart';

void main() {
  test('delegates profile following through the social graph', () async {
    final social = FakeNostrSocialPort();
    final harness = await buildHybridRepositoryHarness(
      FakeRemoteVideoSource([]),
      ports: HybridHarnessPorts(social: social),
    );
    final creator = ProfileId.parse('creator');

    final followed = await harness.profile.toggleFollow(creator);

    expect(followed, isTrue);
    expect(social.followedProfiles, {creator});
  });
}
