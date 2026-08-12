import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/profile/domain/profile_metadata_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_profile_repository.dart';
import 'package:ghostr/features/video_catalog/domain/toggle_profile_follow_workflow.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_screen.dart';

import 'fake_activity_repository.dart';
import 'recording_failure_reporter.dart';

Widget profileScreenHarness({
  required VideoProfileRepository profile,
  required ProfileSummary viewer,
  required ProfileId profileId,
  VoidCallback? onSignedOut,
  ProfileMetadataRepository? metadata,
  ValueChanged<ProfileSummary>? onCurrentProfileUpdated,
  ValueChanged<VideoPost>? onOpenVideo,
}) {
  final cubit = ProfileCubit(
    ProfileDependencies(
      profile: profile,
      toggleFollow: DefaultToggleProfileFollowWorkflow(
        profile: profile,
        activity: FakeActivityRepository(),
        clock: () => DateTime(2026, 8, 2),
        failureReporter: RecordingFailureReporter(),
      ),
      metadata: metadata,
      onCurrentProfileUpdated: onCurrentProfileUpdated,
    ),
    ProfileRequest(viewer: viewer, profileId: profileId),
  );
  return MaterialApp(
    home: BlocProvider.value(
      value: cubit..load(),
      child: ProfileScreen(
        onSignedOut: onSignedOut ?? () {},
        onOpenVideo: onOpenVideo,
      ),
    ),
  );
}
