import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/domain/follow_outcome.dart';
import 'package:ghostr/features/video_catalog/domain/follow_profile_workflow.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('does not record activity for an already-followed creator', () async {
    final activity = FakeActivityRepository();
    final workflow = DefaultFollowProfileWorkflow(
      social: _ExistingFollowSocial(),
      activity: activity,
      clock: () => DateTime.utc(2026, 8, 12),
      failureReporter: RecordingFailureReporter(),
    );

    final outcome = await workflow.follow(sampleCreator());

    expect(outcome, FollowOutcome.alreadyFollowing);
    expect(await activity.load(), isEmpty);
  });
}

final class _ExistingFollowSocial extends FakeVideoCatalogRepository {
  _ExistingFollowSocial() : super(forYouFeed: []);

  @override
  Future<FollowOutcome> follow(ProfileId profileId) async {
    return FollowOutcome.alreadyFollowing;
  }
}
