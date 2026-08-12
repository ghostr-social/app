import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/domain/profile_metadata_repository.dart';
import 'package:ghostr/features/profile/domain/profile_image_workflow.dart';
import 'package:ghostr/features/profile/domain/selected_profile_image.dart';
import 'package:ghostr/features/profile/presentation/profile_edit_state.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';

export 'profile_edit_state.dart';

final class ProfileEditCubit extends DisposalSafeCubit<ProfileEditState> {
  ProfileEditCubit(
    this._repository,
    this._identity, [
    this._images = const ProfileImageWorkflow.disabled(),
  ]) : super(const ProfileEditIdle());

  final ProfileMetadataRepository _repository;
  final NostrIdentity _identity;
  final ProfileImageWorkflow _images;

  Future<void> selectPicture() async {
    if (state is ProfileEditSelectingPicture || state is ProfileEditSaving) {
      return;
    }
    final previous = state.selectedPicture;
    emit(ProfileEditSelectingPicture(selectedPicture: previous));
    try {
      final selected = await _images.select();
      emit(ProfileEditIdle(selectedPicture: selected ?? previous));
    } on AppFailure catch (failure) {
      emit(ProfileEditFailure(failure.message, selectedPicture: previous));
    } on Object catch (error, stackTrace) {
      _emitFailure(error, stackTrace, previous);
    }
  }

  Future<void> save(ProfileMetadata metadata) async {
    if (state is ProfileEditSaving ||
        state is ProfileEditSelectingPicture ||
        state is ProfileEditSaved) {
      return;
    }
    final selected = state.selectedPicture;
    emit(ProfileEditSaving(selectedPicture: selected));
    try {
      final resolved = await _images.resolve(metadata, selected);
      emit(ProfileEditSaved(await _repository.save(_identity, resolved)));
    } on AppFailure catch (failure) {
      emit(ProfileEditFailure(failure.message, selectedPicture: selected));
    } on Object catch (error, stackTrace) {
      _emitFailure(error, stackTrace, selected);
    }
  }

  void _emitFailure(
    Object error,
    StackTrace stackTrace,
    SelectedProfileImage? selected,
  ) {
    final failure = translatedBoundaryFailure(
      source: 'ProfileEditCubit',
      message: 'Could not save this profile.',
      error: error,
      stackTrace: stackTrace,
    );
    emit(ProfileEditFailure(failure.message, selectedPicture: selected));
  }
}
