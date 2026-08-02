import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details_policy.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/sample_data.dart';

void main() {
  test('uses the viewer and follow count for an empty current-user profile',
      () {
    final viewer = sampleCreator(id: 'viewer');

    final details = const ProfileDetailsPolicy().build(
      ProfileSocialContext(
        viewer: viewer,
        targetId: viewer.id,
        followed: {ProfileId.parse('followed')},
        blocked: const {},
      ),
      const [],
    );

    expect(details.profile, same(viewer));
    expect(details.followingCount, 1);
    expect(details.isCurrentUser, isTrue);
  });
}
