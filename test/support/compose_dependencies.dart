import 'package:ghostr/core/media/media_picker_port.dart';
import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/compose/domain/publish_video_workflow.dart';
import 'package:ghostr/features/compose/presentation/compose_cubit.dart';
import 'package:ghostr/features/publish/domain/video_publishing_repository.dart';

import 'recording_failure_reporter.dart';

ComposeDependencies buildComposeDependencies({
  required VideoPublishingRepository publishing,
  required ActivityRepository activity,
  required MediaPickerPort picker,
  PublishVideoClock? clock,
  FailureReporter? failureReporter,
}) {
  return ComposeDependencies(
    publishVideo: DefaultPublishVideoWorkflow(
      publishing: publishing,
      activity: activity,
      clock: clock ?? () => DateTime(2026, 8, 2),
      failureReporter: failureReporter ?? RecordingFailureReporter(),
    ),
    mediaPicker: picker,
  );
}
