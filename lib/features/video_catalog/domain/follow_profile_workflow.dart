import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/activity/domain/activity_type.dart';
import 'package:ghostr/features/social/domain/follow_outcome.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

abstract interface class FollowProfileWorkflow {
  Future<FollowOutcome> follow(ProfileSummary profile);
}

final class DefaultFollowProfileWorkflow implements FollowProfileWorkflow {
  const DefaultFollowProfileWorkflow({
    required SocialGraphRepository social,
    required ActivityRepository activity,
    required DateTime Function() clock,
    required FailureReporter failureReporter,
  }) : _social = social,
       _activity = activity,
       _clock = clock,
       _failureReporter = failureReporter;

  final SocialGraphRepository _social;
  final ActivityRepository _activity;
  final DateTime Function() _clock;
  final FailureReporter _failureReporter;

  @override
  Future<FollowOutcome> follow(ProfileSummary profile) async {
    final activity = _activity.snapshotForActiveAccount();
    final outcome = await _social.follow(profile.id);
    if (outcome == FollowOutcome.newlyFollowed) {
      await _record(profile, activity);
    }
    return outcome;
  }

  Future<void> _record(
    ProfileSummary profile,
    ActivityRepository activity,
  ) async {
    try {
      await activity.record(_activityItem(profile));
    } on Object catch (error, stackTrace) {
      _failureReporter.report(
        source: 'DefaultFollowProfileWorkflow.record',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }

  ActivityItem _activityItem(ProfileSummary profile) {
    final occurredAt = _clock();
    return ActivityItem(
      id: ActivityId.parse(
        'follow-${profile.id.value}-${occurredAt.millisecondsSinceEpoch}',
      ),
      type: ActivityType.follow,
      description: ActivityDescription(
        title: 'Started following ${profile.displayName}',
        body: profile.handle,
      ),
      occurredAt: occurredAt,
    );
  }
}
