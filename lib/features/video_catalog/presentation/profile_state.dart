import 'package:ghostr/features/video_catalog/domain/profile_details.dart';

enum ProfileStatus { loading, ready, failure }

sealed class ProfileState {
  const ProfileState();

  const factory ProfileState.loading() = ProfileLoading;

  const factory ProfileState.failure(String message) = ProfileFailure;

  factory ProfileState.ready(
    ProfileDetails details, {
    bool isUpdating = false,
    bool isRefreshing = false,
    String? notice,
    String? refreshError,
  }) {
    return ProfileReady(
      details,
      isUpdating: isUpdating,
      isRefreshing: isRefreshing,
      notice: notice,
      refreshError: refreshError,
    );
  }

  ProfileStatus get status;
  ProfileDetails? get details => null;
  bool get isUpdating => false;
  bool get isRefreshing => false;
  String? get message => null;
  String? get notice => null;
  String? get refreshError => null;
}

final class ProfileLoading extends ProfileState {
  const ProfileLoading();

  @override
  ProfileStatus get status => ProfileStatus.loading;
}

final class ProfileFailure extends ProfileState {
  const ProfileFailure(this.failureMessage);

  final String failureMessage;

  @override
  ProfileStatus get status => ProfileStatus.failure;

  @override
  String get message => failureMessage;
}

final class ProfileReady extends ProfileState {
  const ProfileReady(
    this.readyDetails, {
    this.isUpdating = false,
    this.isRefreshing = false,
    this.notice,
    this.refreshError,
  });

  final ProfileDetails readyDetails;
  @override
  final bool isUpdating;
  @override
  final bool isRefreshing;
  @override
  final String? notice;
  @override
  final String? refreshError;

  @override
  ProfileStatus get status => ProfileStatus.ready;

  @override
  ProfileDetails get details => readyDetails;

  ProfileReady transition(ProfileReadyTransition change) {
    return ProfileReady(
      change.details ?? readyDetails,
      isUpdating: change.isUpdating ?? isUpdating,
      isRefreshing: change.isRefreshing ?? isRefreshing,
      notice: change.clearNotice ? null : change.notice ?? notice,
      refreshError: change.clearRefreshError
          ? null
          : change.refreshError ?? refreshError,
    );
  }

  ProfileReady withoutNotice() {
    return transition(const ProfileReadyTransition(clearNotice: true));
  }
}

final class ProfileReadyTransition {
  const ProfileReadyTransition({
    this.details,
    this.isUpdating,
    this.isRefreshing,
    this.notice,
    this.refreshError,
    this.clearNotice = false,
    this.clearRefreshError = false,
  });

  final ProfileDetails? details;
  final bool? isUpdating;
  final bool? isRefreshing;
  final String? notice;
  final String? refreshError;
  final bool clearNotice;
  final bool clearRefreshError;
}
