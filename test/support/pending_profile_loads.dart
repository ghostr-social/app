import 'dart:async';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/toggle_profile_follow_workflow.dart';
import 'package:ghostr/features/video_catalog/domain/video_profile_repository.dart';

final class PendingProfileLoads implements VideoProfileRepository {
  PendingProfileLoads({this.initial});

  final ProfileDetails? initial;
  final pending = Completer<ProfileDetails>();
  var loadCount = 0;

  @override
  Future<ProfileDetails> loadProfile(
    ProfileSummary viewer,
    ProfileId profileId,
  ) {
    loadCount += 1;
    if (loadCount == 1 && initial != null) return Future.value(initial);
    return pending.future;
  }

  @override
  Future<bool> toggleBlock(ProfileId profileId) async => false;

  @override
  Future<bool> toggleFollow(ProfileId profileId) async => false;
}

final class UnusedProfileFollow implements ToggleProfileFollowWorkflow {
  @override
  Future<ToggleProfileFollowOutcome> toggle(ProfileDetails details) {
    throw UnimplementedError();
  }
}

final class FailingProfileLoads implements VideoProfileRepository {
  var loadCount = 0;

  @override
  Future<ProfileDetails> loadProfile(
    ProfileSummary viewer,
    ProfileId profileId,
  ) async {
    loadCount += 1;
    throw const AppFailure('Profile refresh failed.');
  }

  @override
  Future<bool> toggleBlock(ProfileId profileId) async => false;

  @override
  Future<bool> toggleFollow(ProfileId profileId) async => false;
}
