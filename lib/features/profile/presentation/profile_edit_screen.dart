import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/profile/presentation/profile_edit_cubit.dart';
import 'package:ghostr/features/profile/presentation/profile_metadata_form_screen.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

final class ProfileEditScreen extends StatelessWidget {
  const ProfileEditScreen({required this.initial, super.key});

  final ProfileSummary initial;

  @override
  Widget build(BuildContext context) {
    return BlocConsumer<ProfileEditCubit, ProfileEditState>(
      listenWhen: (_, state) => state is ProfileEditSaved,
      listener: _closeWithSavedProfile,
      builder: (context, state) => ProfileMetadataFormScreen(
        configuration: ProfileMetadataFormConfiguration(
          initial: ProfileFormInitial.fromSummary(initial),
          title: 'Edit profile',
          submitLabel: 'Save profile',
        ),
        actions: ProfileMetadataFormActions(
          onSelectPicture: context.read<ProfileEditCubit>().selectPicture,
          onSubmit: context.read<ProfileEditCubit>().save,
        ),
        viewState: ProfileMetadataFormViewState(
          isSubmitting: state is ProfileEditSaving,
          errorMessage: state.message,
          selectedPicture: state.selectedPicture,
          isSelectingPicture: state is ProfileEditSelectingPicture,
        ),
      ),
    );
  }

  void _closeWithSavedProfile(BuildContext context, ProfileEditState state) {
    Navigator.of(context).pop((state as ProfileEditSaved).profile);
  }
}
