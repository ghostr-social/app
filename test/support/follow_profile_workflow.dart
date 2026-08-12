import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/follow_profile_workflow.dart';

import 'fake_activity_repository.dart';
import 'recording_failure_reporter.dart';

FollowProfileWorkflow testFollowProfileWorkflow(SocialGraphRepository social) {
  return DefaultFollowProfileWorkflow(
    social: social,
    activity: FakeActivityRepository(),
    clock: () => DateTime.utc(2026, 8, 12),
    failureReporter: RecordingFailureReporter(),
  );
}
