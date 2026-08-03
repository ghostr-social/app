import 'package:ghostr/features/video_catalog/domain/profile_details.dart';

enum ProfileStatus { loading, ready, failure }

sealed class ProfileState {
  const ProfileState();

  const factory ProfileState.loading() = ProfileLoading;

  const factory ProfileState.failure(String message) = ProfileFailure;

  factory ProfileState.ready(
    ProfileDetails details, {
    bool isUpdating = false,
    String? notice,
  }) {
    return ProfileReady(details, isUpdating: isUpdating, notice: notice);
  }

  ProfileStatus get status;
  ProfileDetails? get details => null;
  bool get isUpdating => false;
  String? get message => null;
  String? get notice => null;
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
    this.notice,
  });

  final ProfileDetails readyDetails;
  @override
  final bool isUpdating;
  @override
  final String? notice;

  @override
  ProfileStatus get status => ProfileStatus.ready;

  @override
  ProfileDetails get details => readyDetails;

  ProfileReady withoutNotice() => ProfileReady(readyDetails);
}
