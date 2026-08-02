import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/toggle_profile_follow_workflow.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('records activity when a profile becomes followed', () async {
    final profile = FakeVideoCatalogRepository(forYouFeed: []);
    final activity = FakeActivityRepository();
    final reporter = RecordingFailureReporter();
    final workflow = DefaultToggleProfileFollowWorkflow(
      profile: profile,
      activity: activity,
      clock: () => DateTime.utc(2026, 8, 2),
      failureReporter: reporter,
    );

    final notice = await workflow.toggle(sampleProfileDetails());

    expect(notice, ToggleProfileFollowOutcome.followed);
    expect(reporter.sources, isEmpty);
    expect((await activity.load()).single.type.name, 'follow');
  });
}
