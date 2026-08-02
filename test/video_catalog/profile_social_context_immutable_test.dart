import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details_policy.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/sample_data.dart';

void main() {
  test('does not expose mutable profile relationship inputs', () {
    final followed = <ProfileId>{ProfileId.parse('followed')};
    final context = ProfileSocialContext(
      viewer: sampleCreator(),
      targetId: ProfileId.parse('target'),
      followed: followed,
      blocked: const {},
    );
    followed.clear();

    expect(context.followed, hasLength(1));
    expect(() => context.followed.clear(), throwsUnsupportedError);
    expect(() => context.blocked.add(ProfileId.parse('blocked')),
        throwsUnsupportedError);
  });
}
