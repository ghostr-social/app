import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/toggle_profile_follow_workflow.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  final creator = sampleCreator();
  final activity = FakeActivityRepository();
  final catalog = FakeVideoCatalogRepository(
    forYouFeed: [samplePost(creator: creator)],
    feed: FakeFeedScenario(profiles: {
      creator.id: sampleProfileDetails(profile: creator),
    }),
  );
  blocTest<ProfileCubit, ProfileState>(
    'publishes a follow and records it in device activity',
    build: () => ProfileCubit(
      ProfileDependencies(
        profile: catalog,
        toggleFollow: DefaultToggleProfileFollowWorkflow(
          profile: catalog,
          activity: activity,
          clock: () => DateTime(2026, 8, 2),
          failureReporter: RecordingFailureReporter(),
        ),
      ),
      ProfileRequest(viewer: sampleSession().profile, profileId: creator.id),
    ),
    act: (cubit) async {
      await cubit.load();
      await cubit.toggleFollow();
    },
    verify: (cubit) async {
      final item = (await activity.load()).single;
      expect(item.title, 'Started following ${creator.displayName}');
      expect(item.body, creator.handle);
    },
  );
}
