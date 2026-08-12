import 'package:flutter/foundation.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/domain/selected_profile_image.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

final class ProfileFormInitial {
  const ProfileFormInitial({
    required this.displayName,
    required this.handle,
    required this.pictureUrl,
  });

  factory ProfileFormInitial.fromMetadata(ProfileMetadata? metadata) {
    return ProfileFormInitial(
      displayName: metadata?.displayName.value ?? '',
      handle: metadata?.handle.value ?? '',
      pictureUrl: metadata?.pictureUrl?.value ?? '',
    );
  }

  factory ProfileFormInitial.fromSummary(ProfileSummary summary) {
    return ProfileFormInitial(
      displayName: summary.displayName,
      handle: summary.handle,
      pictureUrl: summary.avatarUrl ?? '',
    );
  }

  final String displayName;
  final String handle;
  final String pictureUrl;
}

final class ProfileMetadataFormConfiguration {
  const ProfileMetadataFormConfiguration({
    required this.initial,
    required this.title,
    required this.submitLabel,
    this.submitKey = const Key('profile-form-submit'),
  });

  final ProfileFormInitial initial;
  final String title;
  final String submitLabel;
  final Key submitKey;
}

final class ProfileMetadataFormActions {
  const ProfileMetadataFormActions({
    required this.onSubmit,
    this.onSelectPicture,
    this.onBack,
  });

  final ValueChanged<ProfileMetadata> onSubmit;
  final VoidCallback? onSelectPicture;
  final VoidCallback? onBack;
}

final class ProfileMetadataFormViewState {
  const ProfileMetadataFormViewState({
    this.isSubmitting = false,
    this.errorMessage,
    this.selectedPicture,
    this.isSelectingPicture = false,
  });

  final bool isSubmitting;
  final String? errorMessage;
  final SelectedProfileImage? selectedPicture;
  final bool isSelectingPicture;
}
