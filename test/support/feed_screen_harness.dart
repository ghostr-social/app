import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_workflow.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import 'fake_media_ports.dart';
import 'fake_video_sharing.dart';
import 'fake_video_catalog_repository.dart';

Widget feedScreenHarness(
  FakeVideoCatalogRepository repository, {
  ValueChanged<String>? onOpenProfile,
  ValueChanged<String>? onOpenHashtag,
  VideoPlaybackPort? playbackPort,
  VideoShareWorkflow? shareWorkflow,
}) {
  return MaterialApp(
    home: BlocProvider(
      create: (_) => FeedCubit(
        FeedDependencies(
          feed: repository,
          engagement: repository,
          optional: FeedOptionalDependencies(social: repository),
        ),
      )..load(),
      child: Scaffold(
        body: FeedScreen(
          bindings: FeedScreenBindings(
            onOpenProfile: onOpenProfile ?? (_) {},
            onOpenHashtag: onOpenHashtag ?? (_) {},
            playbackPort: playbackPort ?? FakeVideoPlaybackPort(),
            shareWorkflow: shareWorkflow ?? FakeVideoShareWorkflow(),
            createComments: (post) => CommentsCubit(repository, post),
            isActive: true,
          ),
        ),
      ),
    ),
  );
}
