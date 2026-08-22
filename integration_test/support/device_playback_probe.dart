import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';
import 'package:ghostr/features/video_inventory/domain/playback_telemetry_port.dart';

part 'device_playback_probe_models.dart';

final class DevicePlaybackProbe implements PlaybackTelemetryPort {
  final _watch = Stopwatch()..start();
  final activations = <PlaybackSession>[];
  final deactivations = <PlaybackSession>[];
  final observations = <TimedPlaybackObservation>[];
  final presentations = <PlaybackSession>[];
  var _nextGeneration = 0;

  Duration get elapsed => _watch.elapsed;

  PlaybackFocus markFocus(PlaybackVideoId videoId) =>
      PlaybackFocus(videoId, elapsed);

  @override
  PlaybackSession openSession(
    PlaybackVideoId videoId,
    PlaybackDeliveryId deliveryId,
  ) => PlaybackSession(videoId, deliveryId, ++_nextGeneration);

  @override
  void activate(PlaybackSession session) => activations.add(session);

  @override
  void report(PlaybackObservation observation) {
    observations.add(TimedPlaybackObservation(elapsed, observation));
  }

  @override
  void presented(PlaybackSession session) => presentations.add(session);

  @override
  void deactivate(PlaybackSession session) => deactivations.add(session);

  Duration? playingLatency(PlaybackFocus focus) {
    final event = _firstPhaseAfter(
      PlaybackPhase.playing,
      focus.startedAt,
      videoId: focus.videoId,
    );
    return event == null ? null : event.elapsed - focus.startedAt;
  }

  Duration? recoveryLatency(Duration releasedAt) {
    final event = _firstPhaseAfter(PlaybackPhase.playing, releasedAt);
    return event == null ? null : event.elapsed - releasedAt;
  }

  Duration latestPosition(PlaybackVideoId videoId) {
    var result = Duration.zero;
    for (final event in observations) {
      if (event.observation.videoId == videoId) {
        result = event.observation.position;
      }
    }
    return result;
  }

  bool hasPhaseAfter(PlaybackPhase phase, Duration after) {
    return _firstPhaseAfter(phase, after) != null;
  }

  double get rebufferRatio {
    final first = observations.indexWhere(_isPlaying);
    if (first < 0) return double.infinity;
    final observed = elapsed - observations[first].elapsed;
    if (observed == Duration.zero) return double.infinity;
    return _stalledDuration(first).inMicroseconds / observed.inMicroseconds;
  }

  Duration _stalledDuration(int first) {
    var result = Duration.zero;
    for (var index = first; index < observations.length; index += 1) {
      if (observations[index].observation.phase !=
          PlaybackPhase.networkStalled) {
        continue;
      }
      final end = index + 1 < observations.length
          ? observations[index + 1].elapsed
          : elapsed;
      result += end - observations[index].elapsed;
    }
    return result;
  }

  TimedPlaybackObservation? _firstPhaseAfter(
    PlaybackPhase phase,
    Duration after, {
    PlaybackVideoId? videoId,
  }) {
    for (final event in observations) {
      if (event.elapsed < after || event.observation.phase != phase) continue;
      if (videoId == null || event.observation.videoId == videoId) return event;
    }
    return null;
  }
}
