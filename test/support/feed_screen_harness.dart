import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation_updates.dart';
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
    this.reposts,
    this.feed,
    this.focus,
    this.preparationUpdates,
    this.watch = const FeedWatchDependencies(),
  });

  final ValueChanged<String>? onOpenProfile;
  final ValueChanged<String>? onOpenHashtag;
  final VideoPlaybackPort? playbackPort;
  final VideoShareWorkflow? shareWorkflow;
  final ProfileId? viewerId;
  final SocialGraphRepository? social;
  final VideoRepostRepository? reposts;
  final VideoFeedRepository? feed;
  final FeedFocusPort? focus;
  final PlaybackPreparationUpdates? preparationUpdates;
  final FeedWatchDependencies watch;
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
          feed: options.feed ?? repository,
          engagement: repository,
          followProfile: testFollowProfileWorkflow(socialGraph),
          optional: FeedOptionalDependencies(
            social: socialGraph,
            focus: options.focus,
            watch: options.watch,
            delivery: FeedDeliveryDependencies(
              reposts: options.reposts ?? repository,
              preparationUpdates: options.preparationUpdates,
            ),
          ),
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
            showFeedKindSelector: true,
          ),
        ),
      ),
    ),
  );
}
