part of 'warp_feed_production_graph.dart';

typedef WarpFeedProductionComposition = ({
  AppDependencies dependencies,
  ProductionVideoDelivery delivery,
  Ndk ndk,
  ProgressiveDeviceTelemetry telemetry,
  WarpFeedPreparationMetrics preparation,
  WarpFeedRustProbe rustProbe,
});

typedef _WarpFeedRuntime = ({
  AppDependencies dependencies,
  ProductionVideoDelivery delivery,
  Ndk ndk,
  AppControllerFactory controllers,
  FeedCubit cubit,
  VideoPlaybackPort playback,
});

typedef _WarpFeedEvidence = ({
  ProgressiveDeviceTelemetry telemetry,
  WarpFeedPlayerStageProbe playerStages,
  WarpFeedPreparationMetrics preparation,
  WarpFeedFocusProbe focus,
  WarpFeedRustProbe rustProbe,
});
