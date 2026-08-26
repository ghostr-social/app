import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';
import 'package:ghostr/features/video_inventory/domain/playback_telemetry_port.dart';

part 'device_playback_probe_models.dart';
part 'device_playback_probe_queries.dart';
part 'device_playback_probe_rebuffer.dart';

final class DevicePlaybackProbe implements PlaybackTelemetryPort {
  final _watch = Stopwatch()..start();
  final activations = <PlaybackSession>[];
  final deactivations = <PlaybackSession>[];
  final observations = <TimedPlaybackObservation>[];
  final presentations = <PlaybackSession>[];
  final _focuses = <PlaybackFocus>[];
  final _ownershipEvents = <TimedPlaybackOwnership>[];
  var _evidenceSequence = 0;
  var _nextGeneration = 0;

  Duration get elapsed => _watch.elapsed;

  PlaybackFocus markFocus(
    PlaybackVideoId videoId, {
    FeedFocusCause cause = FeedFocusCause.userNavigation,
    FeedTransportRescue? rescue,
  }) {
    final focus = PlaybackFocus(
      videoId,
      elapsed,
      markExternalEvidence(),
      cause,
      rescue,
    );
    _focuses.add(focus);
    return focus;
  }

  int markExternalEvidence() => ++_evidenceSequence;

  @override
  PlaybackSession openSession(
    PlaybackVideoId videoId,
    PlaybackDeliveryId deliveryId,
  ) => PlaybackSession(videoId, deliveryId, ++_nextGeneration);

  @override
  void activate(PlaybackSession session) {
    activations.add(session);
    _recordOwnership(session, PlaybackOwnershipAction.activate);
  }

  @override
  void report(PlaybackObservation observation) {
    observations.add(
      TimedPlaybackObservation(elapsed, observation, markExternalEvidence()),
    );
  }

  @override
  void presented(PlaybackSession session) {
    presentations.add(session);
    _recordOwnership(session, PlaybackOwnershipAction.presented);
  }

  @override
  void deactivate(PlaybackSession session) {
    deactivations.add(session);
    _recordOwnership(session, PlaybackOwnershipAction.deactivate);
  }

  void _recordOwnership(
    PlaybackSession session,
    PlaybackOwnershipAction action,
  ) {
    _ownershipEvents.add(
      TimedPlaybackOwnership(elapsed, session, action, markExternalEvidence()),
    );
  }
}
