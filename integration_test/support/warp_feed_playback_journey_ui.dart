part of 'warp_feed_playback_journey.dart';

WarpFeedPlaybackJourney _composeJourney(
  ProgressiveDeviceResources resources,
  WarpFeedRelay relay,
  List<Nip01Event> events,
) {
  final source = RustFeedRemoteSource(port: const FfiRustFeedPort());
  final metrics = WarpFeedPreparationMetrics();
  final preparation = WarpFeedPreparationProbe(
    const FfiPlaybackPreparationUpdates(),
    metrics,
  );
  final delivery = ProductionVideoDelivery(
    ProductionVideoDeliverySources.shared(source),
    preparationUpdates: preparation,
    playbackCapabilities: VideoPlaybackCapabilities.progressiveOnly,
  );
  final telemetry = ProgressiveDeviceTelemetry();
  final focus = WarpFeedFocusProbe(FfiFeedFocusPort(), telemetry.probe);
  final playback = buildProductionVideoPlayback(
    delivery,
    playbackTelemetry: telemetry,
  );
  final cubit = _feedCubit(source, preparation, focus);
  return WarpFeedPlaybackJourney._(
    resources: resources,
    relay: relay,
    events: events,
    cubit: cubit,
    playback: playback,
    telemetry: telemetry,
    preparation: metrics,
    focus: focus,
  );
}

FeedCubit _feedCubit(
  RustFeedRemoteSource source,
  WarpFeedPreparationProbe preparation,
  WarpFeedFocusProbe focus,
) {
  return FeedCubit(
    FeedDependencies(
      feed: WarpRemoteFeedRepository(source),
      engagement: const WarpNoopEngagement(),
      optional: FeedOptionalDependencies(
        focus: focus,
        delivery: FeedDeliveryDependencies(
          deliveryUpdates: FfiVideoDeliveryUpdates(),
          preparationUpdates: preparation,
        ),
      ),
    ),
  );
}

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
            shareWorkflow: const WarpNoopShare(),
            createComments: (post) =>
                CommentsCubit(const WarpNoopComments(), post),
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
