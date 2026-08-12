import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_cubit.dart';

import '../support/pending_profile_metadata_repository.dart';
import '../support/pending_profile_loads.dart';
import '../support/sample_data.dart';

void main() {
  test('profile load completes while relay metadata remains pending', () async {
    final viewer = sampleSession().profile;
    final details = ProfileDetails.empty(viewer);
    final metadata = PendingProfileMetadataRepository();
    final cubit = ProfileCubit(
      ProfileDependencies(
        profile: PendingProfileLoads(initial: details),
        toggleFollow: UnusedProfileFollow(),
        metadata: metadata,
      ),
      ProfileRequest(viewer: viewer, profileId: viewer.id),
    );
    addTearDown(() async {
      metadata.pending.complete(null);
      await cubit.close();
    });

    await cubit.load();

    expect(cubit.state.details, same(details));
    expect(metadata.refreshCount, 1);
  });
}
