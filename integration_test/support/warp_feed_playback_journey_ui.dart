part of 'warp_feed_playback_journey.dart';

extension WarpFeedPlaybackJourneyUi on WarpFeedPlaybackJourney {
  Widget get app => MaterialApp(home: WarpFeedSurface(graph: graph));

  void load() => unawaited(cubit.load());
}
