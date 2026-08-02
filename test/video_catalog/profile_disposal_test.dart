import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/toggle_profile_follow_workflow.dart';
import 'package:ghostr/features/video_catalog/domain/video_profile_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_cubit.dart';

import '../support/sample_data.dart';

void main() {
  test('ignores a profile load completion after disposal', () async {
    final repository = _PendingProfileRepository();
    final cubit = ProfileCubit(
      ProfileDependencies(profile: repository, toggleFollow: _UnusedFollow()),
      ProfileRequest(viewer: sampleCreator(), profileId: sampleCreator().id),
    );

    final load = cubit.load();
    final completion = expectLater(load, completes);
    await cubit.close();
    repository.pending.complete(ProfileDetails.empty(sampleCreator()));

    await completion;
  });
}

class _PendingProfileRepository implements VideoProfileRepository {
  final pending = Completer<ProfileDetails>();

  @override
  Future<ProfileDetails> loadProfile(
    ProfileSummary viewer,
    ProfileId profileId,
  ) =>
      pending.future;

  @override
  Future<bool> toggleBlock(ProfileId profileId) async => false;

  @override
  Future<bool> toggleFollow(ProfileId profileId) async => false;
}

class _UnusedFollow implements ToggleProfileFollowWorkflow {
  @override
  Future<ToggleProfileFollowOutcome> toggle(ProfileDetails details) {
    throw UnimplementedError();
  }
}
