import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/profile/domain/selected_profile_image.dart';

sealed class ProfileEditState {
  const ProfileEditState({this.selectedPicture});

  final SelectedProfileImage? selectedPicture;

  String? get message => null;
}

final class ProfileEditIdle extends ProfileEditState {
  const ProfileEditIdle({super.selectedPicture});
}

final class ProfileEditSaving extends ProfileEditState {
  const ProfileEditSaving({super.selectedPicture});
}

final class ProfileEditSelectingPicture extends ProfileEditState {
  const ProfileEditSelectingPicture({super.selectedPicture});
}

final class ProfileEditFailure extends ProfileEditState {
  const ProfileEditFailure(this.failureMessage, {super.selectedPicture});

  final String failureMessage;

  @override
  String get message => failureMessage;
}

final class ProfileEditSaved extends ProfileEditState {
  const ProfileEditSaved(this.profile);

  final ProfileSummary profile;
}
