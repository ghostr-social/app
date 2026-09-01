part of 'warp_android_lifecycle_scenario.dart';

typedef _LifecyclePlayback = ({
  PlaybackFocus focus,
  PlaybackSession session,
  WarpFeedPlayerStageEvidence stage,
});

extension _WarpAndroidLifecycleQuiescence on WarpAndroidLifecycleScenario {
  Future<void> _teardown(WidgetTester tester) async {
    expect(_playerAttempts, isNotEmpty);
    await tester.pumpWidget(const SizedBox.shrink());
    final limit =
        playbackControllerTeardownTimeout + const Duration(seconds: 2);
    final watch = Stopwatch()..start();
    while (!_isQuiescent && watch.elapsed < limit) {
      await tester.pump(const Duration(milliseconds: 50));
      await Future<void>.delayed(const Duration(milliseconds: 20));
    }
    expect(_isQuiescent, isTrue, reason: _quiescenceEvidence(limit));
    expect(
      videoPlaybackCapacityOf(journey.playback),
      emptyVideoPlaybackCapacitySnapshot,
    );
    debugPrint('WARP_ANDROID_LIFECYCLE_QUIESCENT');
  }

  bool get _isQuiescent =>
      _playerAttempts.every((attempt) => attempt.isTerminal) &&
      videoPlaybackCapacityOf(journey.playback).isQuiescent &&
      journey.resources.origin.activeIncompleteRequestSequences.isEmpty &&
      !journey.resources.origin.headsRemainBlocked;

  List<WarpFeedPlayerStageEvidence> get _playerAttempts {
    final attempts = <WarpFeedPlayerStageEvidence>{};
    for (final event in journey.events) {
      final delivery = journey.focus.deliveryForEvent(event.id);
      if (delivery != null) {
        attempts.addAll(journey.playerStages.attemptsFor(delivery));
      }
    }
    return attempts.toList();
  }

  String _quiescenceEvidence(Duration limit) {
    return 'Android lifecycle teardown exceeded $limit; '
        'attempts=${_playerAttempts.length}, '
        'terminal=${_playerAttempts.where((item) => item.isTerminal).length}, '
        'capacity=${videoPlaybackCapacityOf(journey.playback)}, '
        'active=${journey.resources.origin.activeIncompleteRequestSequences}, '
        'heads=${journey.resources.origin.headsRemainBlocked}.';
  }
}
