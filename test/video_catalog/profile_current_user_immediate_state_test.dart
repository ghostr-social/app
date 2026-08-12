import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_cubit.dart';

import '../support/pending_profile_loads.dart';
import '../support/sample_data.dart';

void main() {
  test('current-user profile is visible before its load completes', () async {
    final viewer = sampleSession().profile;
    final refreshed = ProfileDetails(
      profile: viewer,
      posts: const [],
      statistics: ProfileStatistics(totalLikes: 0, followingCount: 0),
      relationship: ProfileRelationship(
        isFollowing: false,
        isBlocked: false,
        isCurrentUser: true,
      ),
    );
    final repository = PendingProfileLoads();
    final cubit = ProfileCubit(
      ProfileDependencies(
        profile: repository,
        toggleFollow: UnusedProfileFollow(),
      ),
      ProfileRequest(viewer: viewer, profileId: viewer.id),
    );
    Future<void>? loading;
    addTearDown(() async {
      if (!repository.pending.isCompleted) {
        repository.pending.complete(refreshed);
      }
      await loading;
      await cubit.close();
    });

    expect(cubit.state, isA<ProfileReady>());
    expect(cubit.state.details?.profile, same(viewer));

    loading = cubit.load();
    expect(cubit.state, isA<ProfileReady>());
    expect(cubit.state.details?.profile, same(viewer));
  });
}
