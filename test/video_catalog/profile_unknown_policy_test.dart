import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details_policy.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/sample_data.dart';

void main() {
  test('uses an unknown summary for an empty remote profile', () {
    final target = ProfileId.parse('remote');

    final details = const ProfileDetailsPolicy().build(
      ProfileSocialContext(
        viewer: sampleCreator(id: 'viewer'),
        targetId: target,
        followed: const {},
        blocked: const {},
      ),
      const [],
    );

    expect(details.profile.id, target);
    expect(details.profile.displayName, 'Unknown creator');
    expect(details.followingCount, 0);
  });
}
