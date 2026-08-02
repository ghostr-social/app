import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_profile_repository.dart';
import 'package:ghostr/features/video_catalog/domain/toggle_profile_follow_workflow.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_state.dart';

export 'profile_state.dart';

class ProfileDependencies {
  const ProfileDependencies({
    required this.profile,
    required this.toggleFollow,
  });

  final VideoProfileRepository profile;
  final ToggleProfileFollowWorkflow toggleFollow;
}

class ProfileRequest {
  const ProfileRequest({required this.viewer, required this.profileId});

  final ProfileSummary viewer;
  final ProfileId profileId;
}

class ProfileCubit extends DisposalSafeCubit<ProfileState> {
  ProfileCubit(this._dependencies, this._request)
      : super(const ProfileState.loading());

  final ProfileDependencies _dependencies;
  final ProfileRequest _request;

  Future<void> load() async {
    emit(const ProfileState.loading());
    try {
      emit(ProfileState.ready(await _loadDetails()));
    } on AppFailure catch (failure) {
      emit(ProfileState.failure(failure.message));
    } on Object catch (error, stackTrace) {
      final failure = translatedBoundaryFailure(
        source: 'ProfileCubit.load',
        message: 'Could not load this profile.',
        error: error,
        stackTrace: stackTrace,
      );
      emit(ProfileState.failure(failure.message));
    }
  }

  Future<void> toggleFollow() async {
    final details = _updatableDetails;
    if (details == null) return;
    emit(state.updating());
    try {
      final outcome = await _dependencies.toggleFollow.toggle(details);
      emit(ProfileState.ready(
        await _loadDetails(),
        notice: _followNotice(outcome),
      ));
    } on AppFailure catch (failure) {
      emit(ProfileState.ready(details, notice: failure.message));
    } on Object catch (error, stackTrace) {
      final failure = translatedBoundaryFailure(
        source: 'ProfileCubit.toggleFollow',
        message: 'Could not update this follow.',
        error: error,
        stackTrace: stackTrace,
      );
      emit(ProfileState.ready(details, notice: failure.message));
    }
  }

  Future<void> toggleBlock() async {
    final details = _updatableDetails;
    if (details == null) return;
    emit(state.updating());
    try {
      await _dependencies.profile.toggleBlock(details.profile.id);
      emit(ProfileState.ready(await _loadDetails()));
    } on AppFailure catch (failure) {
      emit(ProfileState.ready(details, notice: failure.message));
    } on Object catch (error, stackTrace) {
      final failure = translatedBoundaryFailure(
        source: 'ProfileCubit.toggleBlock',
        message: 'Could not update this block.',
        error: error,
        stackTrace: stackTrace,
      );
      emit(ProfileState.ready(details, notice: failure.message));
    }
  }

  void clearNotice() {
    if (state.notice != null) emit(state.withoutNotice());
  }

  ProfileDetails? get _updatableDetails {
    if (state.status != ProfileStatus.ready || state.isUpdating) return null;
    return state.details;
  }

  Future<ProfileDetails> _loadDetails() {
    return _dependencies.profile.loadProfile(
      _request.viewer,
      _request.profileId,
    );
  }

  String? _followNotice(ToggleProfileFollowOutcome outcome) {
    return outcome == ToggleProfileFollowOutcome.followedWithoutActivity
        ? 'Followed, but local activity history could not be updated.'
        : null;
  }
}
