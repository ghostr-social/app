part of 'warp_feed_playback_journey.dart';

extension WarpFeedPlaybackJourneyAssertionEvidence on WarpFeedPlaybackJourney {
  String _playbackEvidence(PlaybackFocus focus) {
    final session = telemetry.probe.sessionFor(focus);
    final events = telemetry.probe.observations
        .where((event) => event.observation.session == session)
        .map((event) {
          final observation = event.observation;
          return '${observation.phase.name}@${event.elapsed.inMilliseconds}:'
              'position=${observation.position.inMilliseconds}:'
              'buffer=${observation.bufferedExtent.inMilliseconds}';
        })
        .join('|');
    final stages = session == null
        ? ''
        : _playerAttemptEvidence(session.deliveryId);
    return 'video=${focus.videoId.value} focus_ms=${focus.startedAt.inMilliseconds} '
        'delivery=${session?.deliveryId.value} generation=${session?.generation} '
        'events=$events attempts=$stages origin=${_originEvidence()}';
  }

  String _playerAttemptEvidence(PlaybackDeliveryId deliveryId) {
    return playerStages
        .attemptsFor(deliveryId)
        .map((attempt) {
          final asset = attempt.authority.assetId.value;
          return '${asset.substring(0, 8)}:'
              '${attempt.preparedAt.inMilliseconds}/'
              '${attempt.initializingAt?.inMilliseconds}/'
              '${attempt.initializedAt?.inMilliseconds}/'
              '${attempt.firstFrameAt?.inMilliseconds}/'
              '${attempt.releasedAt?.inMilliseconds}/'
              '${attempt.failedAt?.inMilliseconds}';
        })
        .join('|');
  }
}
