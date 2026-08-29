part of 'warp_feed_playback_journey.dart';

extension WarpFeedPlaybackJourneyCadenceAssertions on WarpFeedPlaybackJourney {
  void verifyRapidCadence(Duration startedAt, WarpSwipeBurst burst) {
    final releases = _intervals(startedAt, burst.releases);
    final commits = <Duration>[
      for (var index = 0; index < burst.focuses.length; index += 1)
        burst.focuses[index].startedAt - burst.releases[index],
    ];
    debugPrint(
      'WARP_CADENCE release_ms=${_milliseconds(releases)} '
      'focus_commit_ms=${_milliseconds(commits)}',
    );
    _expectRapid(releases, 'Gesture release cadence');
    _expectRapid(commits, 'Focus commit latency');
  }

  List<Duration> _intervals(Duration start, List<Duration> samples) {
    final intervals = <Duration>[];
    var previous = start;
    for (final sample in samples) {
      intervals.add(sample - previous);
      previous = sample;
    }
    return intervals;
  }

  String _milliseconds(List<Duration> values) {
    return values.map((item) => item.inMilliseconds).join(',');
  }

  void _expectRapid(List<Duration> values, String label) {
    expect(
      deviceRapidCadenceIsWithinTarget(values),
      isTrue,
      reason:
          '$label exceeded '
          '${deviceRapidSwipeMaximumInterval.inMilliseconds} ms.',
    );
  }
}
