part of 'warp_feed_playback_journey.dart';

extension WarpFeedPlaybackJourneyUi on WarpFeedPlaybackJourney {
  Widget get app => MaterialApp(
    home: BlocProvider.value(
      value: cubit,
      child: Scaffold(
        body: FeedScreen(
          bindings: FeedScreenBindings(
            onOpenProfile: (_) {},
            onOpenHashtag: (_) {},
            playbackPort: playback,
            shareWorkflow: graph.dependencies.videoShareWorkflow,
            createComments: graph.controllers.comments,
            isActive: true,
          ),
        ),
      ),
    ),
  );

  PlaybackFocus markFocus(int index) {
    return telemetry.probe.markFocus(PlaybackVideoId.parse(events[index].id));
  }

  void load() => unawaited(cubit.load());
}
