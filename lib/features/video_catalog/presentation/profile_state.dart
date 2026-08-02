import 'package:ghostr/features/video_catalog/domain/profile_details.dart';

enum ProfileStatus { loading, ready, failure }

class ProfileState {
  const ProfileState._({
    required this.status,
    this.details,
    this.isUpdating = false,
    this.message,
    this.notice,
  });

  const ProfileState.loading() : this._(status: ProfileStatus.loading);

  const ProfileState.failure(String message)
      : this._(status: ProfileStatus.failure, message: message);

  const ProfileState.ready(
    ProfileDetails details, {
    bool isUpdating = false,
    String? notice,
  }) : this._(
          status: ProfileStatus.ready,
          details: details,
          isUpdating: isUpdating,
          notice: notice,
        );

  final ProfileStatus status;
  final ProfileDetails? details;
  final bool isUpdating;
  final String? message;
  final String? notice;

  ProfileState updating() {
    return ProfileState.ready(details!, isUpdating: true);
  }

  ProfileState withoutNotice() => ProfileState.ready(details!);
}
