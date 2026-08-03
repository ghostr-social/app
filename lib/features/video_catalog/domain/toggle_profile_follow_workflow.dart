import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/activity/domain/activity_type.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/video_profile_repository.dart';

typedef ProfileFollowClock = DateTime Function();

enum ToggleProfileFollowOutcome {
  followed,
  unfollowed,
  followedWithoutActivity,
}

abstract interface class ToggleProfileFollowWorkflow {
  Future<ToggleProfileFollowOutcome> toggle(ProfileDetails details);
}

class DefaultToggleProfileFollowWorkflow
    implements ToggleProfileFollowWorkflow {
  const DefaultToggleProfileFollowWorkflow({
    required VideoProfileRepository profile,
    required ActivityRepository activity,
    required ProfileFollowClock clock,
    required FailureReporter failureReporter,
  })  : _profile = profile,
        _activity = activity,
        _clock = clock,
        _failureReporter = failureReporter;

  final VideoProfileRepository _profile;
  final ActivityRepository _activity;
  final ProfileFollowClock _clock;
  final FailureReporter _failureReporter;

  @override
  Future<ToggleProfileFollowOutcome> toggle(ProfileDetails details) async {
    final activity = _activity.snapshotForActiveAccount();
    final followed = await _profile.toggleFollow(details.profile.id);
    return followed
        ? _record(details, activity)
        : ToggleProfileFollowOutcome.unfollowed;
  }

  Future<ToggleProfileFollowOutcome> _record(
    ProfileDetails details,
    ActivityRepository activity,
  ) async {
    try {
      await activity.record(_followActivity(details));
      return ToggleProfileFollowOutcome.followed;
    } on Object catch (error, stackTrace) {
      _failureReporter.report(
        source: 'DefaultToggleProfileFollowWorkflow.record',
        error: error,
        stackTrace: stackTrace,
      );
      return ToggleProfileFollowOutcome.followedWithoutActivity;
    }
  }

  ActivityItem _followActivity(ProfileDetails details) {
    final occurredAt = _clock();
    return ActivityItem(
      id: ActivityId.parse(
        'follow-${details.profile.id.value}-${occurredAt.millisecondsSinceEpoch}',
      ),
      type: ActivityType.follow,
      description: ActivityDescription(
        title: 'Started following ${details.profile.displayName}',
        body: details.profile.handle,
      ),
      occurredAt: occurredAt,
    );
  }
}
