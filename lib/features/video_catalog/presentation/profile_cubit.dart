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
  int _generation = 0;

  Future<void> load() async {
    final request = ++_generation;
    emit(const ProfileState.loading());
    try {
      _emitCurrent(request, ProfileState.ready(await _loadDetails()));
    } on AppFailure catch (failure) {
      _emitCurrent(request, ProfileState.failure(failure.message));
    } on Object catch (error, stackTrace) {
      final failure = translatedBoundaryFailure(
        source: 'ProfileCubit.load',
        message: 'Could not load this profile.',
        error: error,
        stackTrace: stackTrace,
      );
      _emitCurrent(request, ProfileState.failure(failure.message));
    }
  }

  Future<void> toggleFollow() async {
    final details = _updatableDetails;
    if (details == null) return;
    final request = _startUpdate(details);
    try {
      final outcome = await _dependencies.toggleFollow.toggle(details);
      _acceptFollow(request, details, outcome);
    } on AppFailure catch (failure) {
      _rejectUpdate(request, details, failure.message);
    } on Object catch (error, stackTrace) {
      _rejectUpdate(request, details, _unexpectedFollow(error, stackTrace));
    }
  }

  Future<void> toggleBlock() async {
    final details = _updatableDetails;
    if (details == null) return;
    final request = _startUpdate(details);
    try {
      final isBlocked =
          await _dependencies.profile.toggleBlock(details.profile.id);
      _emitCurrent(
        request,
        ProfileState.ready(details.copyWith(isBlocked: isBlocked)),
      );
    } on AppFailure catch (failure) {
      _rejectUpdate(request, details, failure.message);
    } on Object catch (error, stackTrace) {
      _rejectUpdate(request, details, _unexpectedBlock(error, stackTrace));
    }
  }

  void clearNotice() {
    final current = state;
    if (current is ProfileReady && current.notice != null) {
      emit(current.withoutNotice());
    }
  }

  ProfileDetails? get _updatableDetails {
    final current = state;
    if (current is! ProfileReady || current.isUpdating) return null;
    return current.details;
  }

  Future<ProfileDetails> _loadDetails() {
    return _dependencies.profile.loadProfile(
      _request.viewer,
      _request.profileId,
    );
  }

  void _emitCurrent(int request, ProfileState next) {
    if (request == _generation) emit(next);
  }

  int _startUpdate(ProfileDetails details) {
    final request = ++_generation;
    emit(ProfileState.ready(details, isUpdating: true));
    return request;
  }

  void _acceptFollow(
    int request,
    ProfileDetails details,
    ToggleProfileFollowOutcome outcome,
  ) {
    _emitCurrent(
      request,
      ProfileState.ready(
        details.copyWith(isFollowing: _isFollowing(outcome)),
        notice: _followNotice(outcome),
      ),
    );
  }

  void _rejectUpdate(int request, ProfileDetails details, String message) {
    _emitCurrent(request, ProfileState.ready(details, notice: message));
  }

  String _unexpectedFollow(Object error, StackTrace stackTrace) {
    return _unexpectedUpdate(
      'ProfileCubit.toggleFollow',
      'Could not update this follow.',
      error,
      stackTrace,
    );
  }

  String _unexpectedBlock(Object error, StackTrace stackTrace) {
    return _unexpectedUpdate(
      'ProfileCubit.toggleBlock',
      'Could not update this block.',
      error,
      stackTrace,
    );
  }

  String _unexpectedUpdate(
    String source,
    String message,
    Object error,
    StackTrace stackTrace,
  ) {
    return translatedBoundaryFailure(
      source: source,
      message: message,
      error: error,
      stackTrace: stackTrace,
    ).message;
  }

  String? _followNotice(ToggleProfileFollowOutcome outcome) {
    return outcome == ToggleProfileFollowOutcome.followedWithoutActivity
        ? 'Followed, but local activity history could not be updated.'
        : null;
  }

  bool _isFollowing(ToggleProfileFollowOutcome outcome) {
    return outcome != ToggleProfileFollowOutcome.unfollowed;
  }
}
