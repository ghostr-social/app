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
  test('a pending follow cannot overwrite a newer profile load', () async {
    final repository = _ProfileRepository();
    final follow = _PendingFollow();
    final cubit = ProfileCubit(
      ProfileDependencies(profile: repository, toggleFollow: follow),
      ProfileRequest(
        viewer: sampleSession().profile,
        profileId: sampleCreator().id,
      ),
    );
    addTearDown(cubit.close);
    await cubit.load();

    final mutation = cubit.toggleFollow();
    final reload = cubit.load();
    repository.refresh.complete(ProfileDetails.empty(sampleSession().profile));
    await reload;
    follow.pending.complete(ToggleProfileFollowOutcome.followed);
    await mutation;

    expect(cubit.state.details?.profile.id, sampleSession().profile.id);
  });
}

class _ProfileRepository implements VideoProfileRepository {
  final refresh = Completer<ProfileDetails>();
  var loads = 0;

  @override
  Future<ProfileDetails> loadProfile(
    ProfileSummary viewer,
    ProfileId profileId,
  ) {
    if (loads++ == 0) {
      return Future.value(ProfileDetails.empty(sampleCreator()));
    }
    return refresh.future;
  }

  @override
  Future<bool> toggleBlock(ProfileId profileId) async => false;

  @override
  Future<bool> toggleFollow(ProfileId profileId) async => true;
}

class _PendingFollow implements ToggleProfileFollowWorkflow {
  final pending = Completer<ToggleProfileFollowOutcome>();

  @override
  Future<ToggleProfileFollowOutcome> toggle(ProfileDetails details) {
    return pending.future;
  }
}
