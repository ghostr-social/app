import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_workflow.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import 'fake_media_ports.dart';
import 'fake_video_sharing.dart';
import 'fake_video_catalog_repository.dart';
import 'follow_profile_workflow.dart';
import 'sample_data.dart';

final class FeedScreenHarnessOptions {
  const FeedScreenHarnessOptions({
    this.onOpenProfile,
    this.onOpenHashtag,
    this.playbackPort,
    this.shareWorkflow,
    this.viewerId,
    this.social,
  });

  final ValueChanged<String>? onOpenProfile;
  final ValueChanged<String>? onOpenHashtag;
  final VideoPlaybackPort? playbackPort;
  final VideoShareWorkflow? shareWorkflow;
  final ProfileId? viewerId;
  final SocialGraphRepository? social;
}

Widget feedScreenHarness(
  FakeVideoCatalogRepository repository, {
  FeedScreenHarnessOptions options = const FeedScreenHarnessOptions(),
}) {
  final socialGraph = options.social ?? repository;
  return MaterialApp(
    home: BlocProvider(
      create: (_) => FeedCubit(
        FeedDependencies(
          viewerId: options.viewerId ?? sampleSession().profile.id,
          feed: repository,
          engagement: repository,
          followProfile: testFollowProfileWorkflow(socialGraph),
          optional: FeedOptionalDependencies(social: socialGraph),
        ),
      )..load(),
      child: Scaffold(
        body: FeedScreen(
          bindings: FeedScreenBindings(
            onOpenProfile: options.onOpenProfile ?? (_) {},
            onOpenHashtag: options.onOpenHashtag ?? (_) {},
            playbackPort: options.playbackPort ?? FakeVideoPlaybackPort(),
            shareWorkflow: options.shareWorkflow ?? FakeVideoShareWorkflow(),
            createComments: (post) => CommentsCubit(repository, post),
            isActive: true,
          ),
        ),
      ),
    ),
  );
}
