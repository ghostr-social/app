import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/activity/domain/activity_type.dart';
import 'package:ghostr/features/publish/domain/video_publishing_repository.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

typedef PublishVideoClock = DateTime Function();

enum PublishVideoOutcome { published, publishedWithoutActivity }

abstract interface class PublishVideoWorkflow {
  Future<PublishVideoOutcome> publish({
    required UserSession session,
    required SelectedMedia media,
    required String rawCaption,
  });
}

class DefaultPublishVideoWorkflow implements PublishVideoWorkflow {
  const DefaultPublishVideoWorkflow({
    required VideoPublishingRepository publishing,
    required ActivityRepository activity,
    required PublishVideoClock clock,
    required FailureReporter failureReporter,
  })  : _publishing = publishing,
        _activity = activity,
        _clock = clock,
        _failureReporter = failureReporter;

  final VideoPublishingRepository _publishing;
  final ActivityRepository _activity;
  final PublishVideoClock _clock;
  final FailureReporter _failureReporter;

  @override
  Future<PublishVideoOutcome> publish({
    required UserSession session,
    required SelectedMedia media,
    required String rawCaption,
  }) async {
    final post = await _publishing.publish(
      session: session,
      media: media,
      caption: _caption(rawCaption, media),
    );
    return _record(post);
  }

  String _caption(String rawCaption, SelectedMedia media) {
    final caption = rawCaption.trim();
    return caption.isEmpty ? media.label : caption;
  }

  Future<PublishVideoOutcome> _record(VideoPost post) async {
    try {
      await _activity.record(_publishedActivity(post));
      return PublishVideoOutcome.published;
    } on Object catch (error, stackTrace) {
      _failureReporter.report(
        source: 'DefaultPublishVideoWorkflow.record',
        error: error,
        stackTrace: stackTrace,
      );
      return PublishVideoOutcome.publishedWithoutActivity;
    }
  }

  ActivityItem _publishedActivity(VideoPost post) {
    return ActivityItem(
      id: ActivityId.parse('publish-${post.id}'),
      type: ActivityType.publish,
      description: ActivityDescription(
        title: 'Published a video',
        body: post.caption,
      ),
      occurredAt: _clock(),
    );
  }
}
