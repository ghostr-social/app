import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_cubit.dart';

import '../support/pending_profile_loads.dart';
import '../support/pending_profile_metadata_repository.dart';
import '../support/sample_data.dart';

void main() {
  test('local self edit wins over an older relay metadata refresh', () async {
    final viewer = sampleSession().profile;
    final metadata = PendingProfileMetadataRepository();
    final loads = PendingProfileLoads();
    final cubit = ProfileCubit(
      ProfileDependencies(
        profile: loads,
        toggleFollow: UnusedProfileFollow(),
        metadata: metadata,
      ),
      ProfileRequest(viewer: viewer, profileId: viewer.id),
    );
    addTearDown(cubit.close);
    final loading = cubit.load();
    final edited = ProfileSummary(
      id: viewer.id,
      displayName: 'Local Nora',
      handle: '@local_nora',
      avatarUrl: null,
    );

    cubit.updateCurrentUser(edited);
    metadata.pending.complete(viewer);
    await Future<void>.delayed(Duration.zero);

    expect(cubit.state.details?.profile, same(edited));
    loads.pending.complete(cubit.state.details!);
    await loading;
  });
}
