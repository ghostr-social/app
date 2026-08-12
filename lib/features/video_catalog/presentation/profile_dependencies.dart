import 'package:ghostr/features/profile/domain/profile_metadata_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/toggle_profile_follow_workflow.dart';
import 'package:ghostr/features/video_catalog/domain/video_profile_repository.dart';

final class ProfileDependencies {
  const ProfileDependencies({
    required this.profile,
    required this.toggleFollow,
    this.metadata,
    this.onCurrentProfileUpdated,
  });

  final VideoProfileRepository profile;
  final ToggleProfileFollowWorkflow toggleFollow;
  final ProfileMetadataRepository? metadata;
  final void Function(ProfileSummary)? onCurrentProfileUpdated;
}
