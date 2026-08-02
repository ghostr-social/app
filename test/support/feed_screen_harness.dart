import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';

import 'fake_media_ports.dart';
import 'fake_video_catalog_repository.dart';

Widget feedScreenHarness(
  FakeVideoCatalogRepository repository, {
  ValueChanged<String>? onOpenProfile,
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
          onOpenProfile: onOpenProfile ?? (_) {},
          playbackPort: FakeVideoPlaybackPort(),
          createComments: (post) => CommentsCubit(repository, post),
        ),
      ),
    ),
  );
}
