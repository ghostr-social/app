import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import 'warp_feed_production_graph.dart';

class WarpFeedSurface extends StatelessWidget {
  const WarpFeedSurface({
    required this.graph,
    this.overlay,
    this.playback,
    super.key,
  });

  final WarpFeedProductionGraph graph;
  final Widget? overlay;
  final VideoPlaybackPort? playback;

  @override
  Widget build(BuildContext context) {
    return BlocProvider.value(
      value: graph.cubit,
      child: Scaffold(body: _body()),
    );
  }

  Widget _body() {
    final feed = FeedScreen(
      bindings: FeedScreenBindings(
        onOpenProfile: (_) {},
        onOpenHashtag: (_) {},
        playbackPort: playback ?? graph.playback,
        shareWorkflow: graph.dependencies.videoShareWorkflow,
        createComments: graph.controllers.comments,
        isActive: true,
      ),
    );
    final badge = overlay;
    return badge == null ? feed : Stack(children: [feed, badge]);
  }
}
