part of 'warp_invalid_track_fallback_scenario.dart';

Future<WarpInvalidTrackFallbackScenario>
_startInvalidTrackFallbackScenario() async {
  final fixture = WarpNoVideoRenditionFixture.install();
  ProgressiveDeviceResources? resources;
  try {
    resources = await ProgressiveDeviceResources.start(
      validator: ProgressiveOriginValidator.stableStrong,
    );
    return await _startInvalidTrackWithResources(resources, fixture);
  } on Object {
    await resources?.close();
    fixture.restore();
    rethrow;
  }
}

Future<WarpInvalidTrackFallbackScenario> _startInvalidTrackWithResources(
  ProgressiveDeviceResources resources,
  WarpNoVideoRenditionFixture fixture,
) async {
  final events = await signedInvalidTrackFallbackEvents(
    resources.origin,
    fixture,
  );
  final relay = await WarpFeedRelay.start(events);
  try {
    final graph = await _buildInvalidTrackGraph(resources, relay);
    return _composeInvalidTrackScenario((
      resources: resources,
      fixture: fixture,
      relay: relay,
      events: events,
      graph: graph,
    ));
  } on Object {
    await relay.close();
    rethrow;
  }
}

Future<WarpFeedProductionGraph> _buildInvalidTrackGraph(
  ProgressiveDeviceResources resources,
  WarpFeedRelay relay,
) {
  return buildWarpFeedProductionGraph(
    resources,
    relay,
    DataUsageLevel.aggressive,
  );
}

WarpInvalidTrackFallbackScenario _composeInvalidTrackScenario(
  _InvalidTrackComposition input,
) {
  final failures = WarpPlayerFailureRecorder(input.graph.playerStages);
  return WarpInvalidTrackFallbackScenario._((
    resources: input.resources,
    fixture: input.fixture,
    relay: input.relay,
    events: input.events,
    graph: input.graph,
    failures: failures,
    playback: _playbackFor(input.graph, failures),
  ));
}

typedef _InvalidTrackComposition = ({
  ProgressiveDeviceResources resources,
  WarpNoVideoRenditionFixture fixture,
  WarpFeedRelay relay,
  List<Nip01Event> events,
  WarpFeedProductionGraph graph,
});

typedef _InvalidTrackRuntime = ({
  ProgressiveDeviceResources resources,
  WarpNoVideoRenditionFixture fixture,
  WarpFeedRelay relay,
  List<Nip01Event> events,
  WarpFeedProductionGraph graph,
  WarpPlayerFailureRecorder failures,
  VideoPlaybackPort playback,
});

VideoPlaybackPort _playbackFor(
  WarpFeedProductionGraph graph,
  WarpPlayerFailureRecorder failures,
) => buildProductionVideoPlayback(
  graph.delivery,
  playbackTelemetry: graph.telemetry,
  playerPreparationFeedback: failures,
);
