import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_cubit.dart';

import '../support/pending_profile_loads.dart';
import '../support/pending_profile_metadata_repository.dart';
import '../support/sample_data.dart';

void main() {
  test('viewing another user does not refresh self metadata', () async {
    final viewer = sampleSession().profile;
    final creator = sampleCreator();
    final metadata = PendingProfileMetadataRepository();
    final cubit = ProfileCubit(
      ProfileDependencies(
        profile: PendingProfileLoads(initial: sampleProfileDetails()),
        toggleFollow: UnusedProfileFollow(),
        metadata: metadata,
      ),
      ProfileRequest(viewer: viewer, profileId: creator.id),
    );
    addTearDown(() async {
      metadata.pending.complete(null);
      await cubit.close();
    });

    await cubit.load();

    expect(metadata.refreshCount, 0);
  });
}
