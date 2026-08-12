import 'package:flutter/material.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/domain/selected_profile_image.dart';
import 'package:ghostr/features/profile/presentation/profile_metadata_form_screen.dart';

final class CreateAccountProfileScreen extends StatelessWidget {
  const CreateAccountProfileScreen({
    required this.initial,
    required this.onSubmit,
    this.selectedPicture,
    this.onSelectPicture,
    this.isSelectingPicture = false,
    this.isSubmitting = false,
    this.errorMessage,
    this.onBack,
    super.key,
  });

  final ProfileMetadata? initial;
  final ValueChanged<ProfileMetadata> onSubmit;
  final SelectedProfileImage? selectedPicture;
  final VoidCallback? onSelectPicture;
  final bool isSelectingPicture;
  final bool isSubmitting;
  final String? errorMessage;
  final VoidCallback? onBack;

  @override
  Widget build(BuildContext context) {
    return ProfileMetadataFormScreen(
      configuration: ProfileMetadataFormConfiguration(
        initial: ProfileFormInitial.fromMetadata(initial),
        title: 'Create your profile',
        submitLabel: 'Create account',
        submitKey: const Key('create-account-submit'),
      ),
      actions: ProfileMetadataFormActions(
        onSubmit: onSubmit,
        onSelectPicture: onSelectPicture,
        onBack: onBack,
      ),
      viewState: ProfileMetadataFormViewState(
        selectedPicture: selectedPicture,
        isSelectingPicture: isSelectingPicture,
        isSubmitting: isSubmitting,
        errorMessage: errorMessage,
      ),
    );
  }
}
