part of 'warp_feed_playback_journey.dart';

extension WarpFeedPlaybackJourneyCadenceAssertions on WarpFeedPlaybackJourney {
  void reportRapidCadence(Duration startedAt, WarpSwipeBurst burst) {
    expect(burst.releases, isNotEmpty, reason: 'Swipe burst was empty.');
    expect(burst.focuses, hasLength(burst.releases.length));
    final releases = _intervals(startedAt, burst.releases);
    final commits = <Duration>[
      for (var index = 0; index < burst.focuses.length; index += 1)
        burst.focuses[index].startedAt - burst.releases[index],
    ];
    _reportCadence(releases, commits);
    _expectCausal(releases, 'Gesture release cadence');
    _expectCausal(commits, 'Focus commit latency');
  }

  void _reportCadence(List<Duration> releases, List<Duration> commits) {
    debugPrint(
      'WARP_CADENCE release_ms=${_milliseconds(releases)} '
      'focus_commit_ms=${_milliseconds(commits)}',
    );
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

  void _expectCausal(List<Duration> values, String label) {
    expect(
      values.every((value) => value >= Duration.zero),
      isTrue,
      reason: '$label contained a negative interval.',
    );
  }
}
