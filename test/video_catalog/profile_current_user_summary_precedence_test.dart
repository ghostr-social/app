import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details_policy.dart';

import '../support/sample_data.dart';

void main() {
  test(
    'current-user profile prefers viewer metadata over stale post metadata',
    () {
      final viewer = sampleCreator(id: 'viewer', displayName: 'Fresh Name');
      final stale = sampleCreator(id: 'viewer', displayName: 'Stale Name');
      final post = samplePost(creator: stale);

      final details = const ProfileDetailsPolicy().build(
        ProfileSocialContext(
          viewer: viewer,
          targetId: viewer.id,
          followed: const {},
          blocked: const {},
        ),
        [post],
      );

      expect(details.profile, same(viewer));
      expect(details.posts, [post]);
    },
  );
}
