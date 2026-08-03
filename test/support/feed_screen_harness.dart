import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import 'fake_media_ports.dart';
import 'fake_video_catalog_repository.dart';

Widget feedScreenHarness(
  FakeVideoCatalogRepository repository, {
  ValueChanged<String>? onOpenProfile,
  VideoPlaybackPort? playbackPort,
}) {
  return MaterialApp(
    home: BlocProvider(
      create: (_) => FeedCubit(FeedDependencies(
        feed: repository,
        engagement: repository,
      ))
        ..load(),
      child: Scaffold(
        body: FeedScreen(
          bindings: FeedScreenBindings(
            onOpenProfile: onOpenProfile ?? (_) {},
            playbackPort: playbackPort ?? FakeVideoPlaybackPort(),
            createComments: (post) => CommentsCubit(repository, post),
            isActive: true,
          ),
        ),
      ),
    ),
  );
}
