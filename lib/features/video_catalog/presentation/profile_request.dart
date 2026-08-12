import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

final class ProfileRequest {
  const ProfileRequest({required this.viewer, required this.profileId});

  final ProfileSummary viewer;
  final ProfileId profileId;
}
