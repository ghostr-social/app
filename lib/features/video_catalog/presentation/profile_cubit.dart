import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/toggle_profile_follow_workflow.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_dependencies.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_follow_projection.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_state.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_initial_state.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_failure_messages.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_metadata_refresh.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_request.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_ready_transitions.dart';

export 'profile_state.dart';
export 'profile_request.dart';
export 'profile_dependencies.dart';

class ProfileCubit extends DisposalSafeCubit<ProfileState> {
  ProfileCubit(this._dependencies, this._request)
    : _metadataRefresh = ProfileMetadataRefresh(_dependencies.metadata),
      super(initialProfileState(_request));

  final ProfileDependencies _dependencies;
  final ProfileMetadataRefresh _metadataRefresh;
  ProfileRequest _request;
  int _generation = 0;
  String? _metadataNotice;
  ProfileSummary? _profileOverride;

  Future<void> load() async {
    final request = ++_generation;
    _metadataNotice = null;
    final current = state;
    if (current is ProfileReady) {
      emit(refreshingProfile(current));
    } else {
      emit(const ProfileState.loading());
    }
    _metadataRefresh.start(
      _request,
      onAccepted: _acceptMetadata,
      onRejected: _rejectMetadata,
    );
    try {
      _emitCurrent(
        request,
        ProfileState.ready(await _loadDetails(), notice: _metadataNotice),
      );
    } on AppFailure catch (failure) {
      _rejectLoad(request, failure.message);
    } on Object catch (error, stackTrace) {
      _rejectLoad(request, unexpectedProfileLoadFailure(error, stackTrace));
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
      _rejectUpdate(
        request,
        details,
        unexpectedProfileFollowFailure(error, stackTrace),
      );
    }
  }

  Future<void> toggleBlock() async {
    final details = _updatableDetails;
    if (details == null) return;
    final request = _startUpdate(details);
    try {
      final isBlocked = await _dependencies.profile.toggleBlock(
        details.profile.id,
      );
      _emitCurrent(
        request,
        ProfileState.ready(details.copyWith(isBlocked: isBlocked)),
      );
    } on AppFailure catch (failure) {
      _rejectUpdate(request, details, failure.message);
    } on Object catch (error, stackTrace) {
      _rejectUpdate(
        request,
        details,
        unexpectedProfileBlockFailure(error, stackTrace),
      );
    }
  }

  void clearNotice() {
    _metadataNotice = null;
    final current = state;
    if (current is ProfileReady && current.notice != null) {
      emit(current.withoutNotice());
    }
  }

  void updateCurrentUser(ProfileSummary profile) {
    final current = state;
    if (current is! ProfileReady || !current.details.isCurrentUser) return;
    if (profile.id != _request.profileId) return;
    _metadataRefresh.cancel();
    _profileOverride = profile;
    _request = ProfileRequest(viewer: profile, profileId: profile.id);
    emit(updatedProfile(current, profile));
  }

  ProfileDetails? get _updatableDetails {
    final current = state;
    if (current is! ProfileReady || current.isUpdating) return null;
    return current.details;
  }

  Future<ProfileDetails> _loadDetails() async {
    final details = await _dependencies.profile.loadProfile(
      _request.viewer,
      _request.profileId,
    );
    return authoritativeProfile(details, _profileOverride);
  }

  void _acceptMetadata(ProfileSummary refreshed) {
    if (isClosed) return;
    final current = state;
    if (current is! ProfileReady || refreshed.id != _request.profileId) {
      return;
    }
    _profileOverride = refreshed;
    _request = ProfileRequest(viewer: refreshed, profileId: refreshed.id);
    emit(updatedProfile(current, refreshed));
    _dependencies.onCurrentProfileUpdated?.call(refreshed);
  }

  void _rejectMetadata(String message) {
    if (isClosed) return;
    _metadataNotice = message;
    final current = state;
    if (current is ProfileReady) {
      emit(current.transition(ProfileReadyTransition(notice: message)));
    }
  }

  void _rejectLoad(int request, String message) {
    final current = state;
    final rejected = current is ProfileReady
        ? rejectedProfileRefresh(current, message)
        : ProfileState.failure(message);
    _emitCurrent(request, rejected);
  }

  void _emitCurrent(int request, ProfileState next) {
    if (request == _generation) emit(next);
  }

  int _startUpdate(ProfileDetails details) {
    _metadataRefresh.cancel();
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
        details.copyWith(isFollowing: isProfileFollowing(outcome)),
        notice: profileFollowNotice(outcome),
      ),
    );
  }

  void _rejectUpdate(int request, ProfileDetails details, String message) {
    _emitCurrent(request, ProfileState.ready(details, notice: message));
  }
}
