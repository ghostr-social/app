import 'package:ghostr/features/video_catalog/domain/toggle_profile_follow_workflow.dart';

String? profileFollowNotice(ToggleProfileFollowOutcome outcome) {
  return outcome == ToggleProfileFollowOutcome.followedWithoutActivity
      ? 'Followed, but local activity history could not be updated.'
      : null;
}

bool isProfileFollowing(ToggleProfileFollowOutcome outcome) {
  return outcome != ToggleProfileFollowOutcome.unfollowed;
}
