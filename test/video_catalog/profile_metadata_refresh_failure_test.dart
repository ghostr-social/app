import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_cubit.dart';

import '../support/fake_profile_metadata_repository.dart';
import '../support/pending_profile_loads.dart';
import '../support/sample_data.dart';

void main() {
  test('relay metadata failure preserves ready cached profile', () async {
    final viewer = sampleSession().profile;
    final metadata = FakeProfileMetadataRepository()
      ..refreshFailure = const AppFailure('Relay metadata unavailable.');
    final cubit = ProfileCubit(
      ProfileDependencies(
        profile: PendingProfileLoads(initial: ProfileDetails.empty(viewer)),
        toggleFollow: UnusedProfileFollow(),
        metadata: metadata,
      ),
      ProfileRequest(viewer: viewer, profileId: viewer.id),
    );
    addTearDown(cubit.close);

    await cubit.load();
    await Future<void>.delayed(Duration.zero);

    expect(cubit.state, isA<ProfileReady>());
    expect(cubit.state.details?.profile, same(viewer));
    expect(cubit.state.notice, 'Relay metadata unavailable.');
  });
}
