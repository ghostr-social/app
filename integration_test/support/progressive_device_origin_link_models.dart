part of 'progressive_device_origin.dart';

typedef _ProgressiveChunk = ({
  int requestSequence,
  String path,
  int start,
  int end,
});

typedef _ProgressiveOriginChunkEventData = ({
  int requestSequence,
  String path,
  int start,
  int end,
  int profileGeneration,
  int bandwidthKbps,
  Duration serviceStartedAt,
  Duration sentAt,
  int sentAtEpochMs,
});

typedef _ProgressiveOriginLinkWindowData = ({
  int generation,
  int bytes,
  Duration duration,
  int achievedBandwidthKbps,
  int confirmedAtEpochMs,
  List<ProgressiveOriginChunkEvent> events,
});

final class ProgressiveOriginChunkEvent {
  ProgressiveOriginChunkEvent._(_ProgressiveOriginChunkEventData data)
    : requestSequence = data.requestSequence,
      path = data.path,
      start = data.start,
      end = data.end,
      profileGeneration = data.profileGeneration,
      bandwidthKbps = data.bandwidthKbps,
      serviceStartedAt = data.serviceStartedAt,
      sentAt = data.sentAt,
      sentAtEpochMs = data.sentAtEpochMs;

  final int requestSequence;
  final String path;
  final int start;
  final int end;
  final int profileGeneration;
  final int bandwidthKbps;
  final Duration serviceStartedAt;
  final Duration sentAt;
  final int sentAtEpochMs;
  int? confirmedAtEpochMs;

  int get bytes => end - start;
}

final class ProgressiveOriginLinkWindow {
  ProgressiveOriginLinkWindow._(_ProgressiveOriginLinkWindowData data)
    : generation = data.generation,
      bytes = data.bytes,
      duration = data.duration,
      achievedBandwidthKbps = data.achievedBandwidthKbps,
      confirmedAtEpochMs = data.confirmedAtEpochMs,
      events = data.events;

  final int generation;
  final int bytes;
  final Duration duration;
  final int achievedBandwidthKbps;
  final int confirmedAtEpochMs;
  final List<ProgressiveOriginChunkEvent> events;
}

final class _ProgressiveOriginChunkPermit {
  _ProgressiveOriginChunkPermit({
    required this.profile,
    required this.serviceStartedAt,
    required this.release,
  });

  final ProgressiveOriginLinkProfile profile;
  final Duration serviceStartedAt;
  final void Function() release;
}
