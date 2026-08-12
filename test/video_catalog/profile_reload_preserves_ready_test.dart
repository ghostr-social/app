import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_cubit.dart';

import '../support/pending_profile_loads.dart';
import '../support/sample_data.dart';

void main() {
  test('reload keeps ready profile details visible while pending', () async {
    final viewer = sampleSession().profile;
    final details = sampleProfileDetails();
    final repository = PendingProfileLoads(initial: details);
    final cubit = ProfileCubit(
      ProfileDependencies(
        profile: repository,
        toggleFollow: UnusedProfileFollow(),
      ),
      ProfileRequest(viewer: viewer, profileId: details.profile.id),
    );
    addTearDown(cubit.close);
    await cubit.load();

    final reload = cubit.load();

    expect(cubit.state, isA<ProfileReady>());
    expect(cubit.state.details, same(details));

    repository.pending.complete(details);
    await reload;
  });
}
