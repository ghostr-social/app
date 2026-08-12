import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/social/domain/follow_outcome.dart';
import 'package:ghostr/features/video_catalog/domain/follow_profile_workflow.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('an activity failure does not undo an accepted feed follow', () async {
    final reporter = RecordingFailureReporter();
    final workflow = DefaultFollowProfileWorkflow(
      social: FakeVideoCatalogRepository(forYouFeed: []),
      activity: _FailingActivity(),
      clock: () => DateTime.utc(2026, 8, 12),
      failureReporter: reporter,
    );

    final outcome = await workflow.follow(sampleCreator());

    expect(outcome, FollowOutcome.newlyFollowed);
    expect(reporter.sources, ['DefaultFollowProfileWorkflow.record']);
  });
}

final class _FailingActivity implements ActivityRepository {
  @override
  ActivityRepository snapshotForActiveAccount() => this;

  @override
  Future<List<ActivityItem>> load() async => const [];

  @override
  Future<void> record(ActivityItem item) => throw StateError('disk full');
}
