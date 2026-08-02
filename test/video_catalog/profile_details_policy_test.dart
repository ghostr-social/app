import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details_policy.dart';

import '../support/sample_data.dart';

void main() {
  test('aggregates a target profile and its social relationship', () {
    final viewer = sampleCreator(id: 'viewer');
    final creator = sampleCreator(id: 'creator');
    final post = samplePost(creator: creator);

    final details = const ProfileDetailsPolicy().build(
      ProfileSocialContext(
        viewer: viewer,
        targetId: creator.id,
        followed: {creator.id},
        blocked: const {},
      ),
      [post],
    );

    expect(details.profile, creator);
    expect(details.totalLikes, post.likeCount);
    expect(details.isFollowing, isTrue);
  });
}
