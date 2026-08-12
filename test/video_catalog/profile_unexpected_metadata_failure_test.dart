import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_cubit.dart';

import '../support/fake_profile_metadata_repository.dart';
import '../support/pending_profile_loads.dart';
import '../support/sample_data.dart';

void main() {
  test('unexpected metadata failure has a safe profile notice', () async {
    final viewer = sampleSession().profile;
    final metadata = FakeProfileMetadataRepository()
      ..refreshFailure = StateError('transport payload');
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

    expect(cubit.state.notice, 'Could not refresh profile details.');
  });
}
