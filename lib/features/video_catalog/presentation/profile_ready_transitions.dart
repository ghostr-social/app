import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_state.dart';

ProfileReady refreshingProfile(ProfileReady current) {
  return current.transition(
    const ProfileReadyTransition(isRefreshing: true, clearRefreshError: true),
  );
}

ProfileReady updatedProfile(ProfileReady current, ProfileSummary profile) {
  return current.transition(
    ProfileReadyTransition(details: current.details.withProfile(profile)),
  );
}

ProfileReady rejectedProfileRefresh(ProfileReady current, String message) {
  return current.transition(
    ProfileReadyTransition(isRefreshing: false, refreshError: message),
  );
}

ProfileDetails authoritativeProfile(
  ProfileDetails details,
  ProfileSummary? override,
) {
  if (override == null ||
      identical(details.profile, override) ||
      details.profile.id != override.id) {
    return details;
  }
  return details.withProfile(override);
}
