import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/video_catalog/domain/toggle_profile_follow_workflow.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('reports local history failure after a successful follow', () async {
    final reporter = RecordingFailureReporter();
    final workflow = DefaultToggleProfileFollowWorkflow(
      profile: FakeVideoCatalogRepository(forYouFeed: []),
      activity: _FailingActivityRepository(),
      clock: () => DateTime.utc(2026, 8, 2),
      failureReporter: reporter,
    );

    final notice = await workflow.toggle(sampleProfileDetails());

    expect(
      notice,
      ToggleProfileFollowOutcome.followedWithoutActivity,
    );
    expect(reporter.sources, ['DefaultToggleProfileFollowWorkflow.record']);
  });
}

class _FailingActivityRepository implements ActivityRepository {
  @override
  ActivityRepository snapshotForActiveAccount() => this;

  @override
  Future<List<ActivityItem>> load() async => const [];

  @override
  Future<void> record(ActivityItem item) => throw StateError('disk full');
}
