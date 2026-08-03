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
  test('ignores an older profile load that completes last', () async {
    final repository = _PendingProfileRepository();
    final cubit = ProfileCubit(
      ProfileDependencies(profile: repository, toggleFollow: _UnusedFollow()),
      ProfileRequest(viewer: sampleCreator(), profileId: sampleCreator().id),
    );
    addTearDown(cubit.close);

    final older = cubit.load();
    final newer = cubit.load();
    final latest = ProfileDetails.empty(sampleCreator());
    repository.pending[1].complete(latest);
    await newer;
    repository.pending[0].complete(
      ProfileDetails.empty(sampleSession().profile),
    );
    await older;

    expect(cubit.state.details, latest);
  });
}

class _PendingProfileRepository implements VideoProfileRepository {
  final pending = <Completer<ProfileDetails>>[];

  @override
  Future<ProfileDetails> loadProfile(
    ProfileSummary viewer,
    ProfileId profileId,
  ) {
    final request = Completer<ProfileDetails>();
    pending.add(request);
    return request.future;
  }

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
