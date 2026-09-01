part of 'warp_long_session_scenario.dart';

const _cancellationVideoId = 'long-06';
const _cancellationPath = '/$_cancellationVideoId.mp4';
const _cancellationBurstCount = 3;
const _unsettledBurstHandoffs = _cancellationBurstCount - 1;

extension _WarpLongSessionBurst on _WarpLongSessionDriver {
  void _armSwipeCancellation() {
    cancellationGate = origin.holdBeforeFirstBody({
      _cancellationPath,
    }, timeout: const Duration(seconds: 45));
  }

  Future<void> _swipeCancellationBurst() async {
    await _wait(() => cancellationGate.isReached);
    cancellationRequest = _exactGatedRequest();
    await _captureCancellationDecisionSequence();
    final expected = _expectedFocuses(
      _LongSwipeDirection.backward,
      _cancellationBurstCount,
    );
    final after = graph.focus.occurrences.last.sequence;
    for (var index = 0; index < expected.length; index += 1) {
      await _gesture(_LongSwipeDirection.backward);
    }
    await _wait(() => _hasFocusOrder(expected, after));
    final focuses = _userFocusesAfter(after).take(expected.length).toList();
    _recordBurst(focuses);
    await _releaseAndExpectCancellation();
    await _waitForDecodedPlayback(focuses.last);
    decodedHandoffs += 1;
    await _expectPlaybackAdvances(focuses.last);
  }

  ProgressiveOriginRequest _exactGatedRequest() {
    final requests = origin
        .requestsFor(_cancellationVideoId)
        .where((request) => request.method == 'GET')
        .toList();
    expect(requests, hasLength(1));
    return requests.single;
  }

  bool _hasFocusOrder(List<String> expected, int after) {
    final observed = _userFocusesAfter(after).take(expected.length).toList();
    if (observed.length != expected.length) return false;
    for (var index = 0; index < expected.length; index += 1) {
      if (observed[index].videoId.value != expected[index]) return false;
    }
    return true;
  }

  void _recordBurst(List<PlaybackFocus> focuses) {
    visited.addAll(focuses.map((focus) => focus.videoId.value));
    handoffs += focuses.length;
  }

  Future<void> _releaseAndExpectCancellation() async {
    await _waitForNativeCancellationEvidence();
    await _waitForCancellationPeerClose();
    cancellationGate.release();
    await _wait(
      () =>
          cancellationRequest.outcome !=
          ProgressiveOriginRequestOutcome.serving,
    );
    expect(
      cancellationRequest.outcome,
      ProgressiveOriginRequestOutcome.clientCanceled,
    );
    expect(
      cancellationRequest.servedBytes,
      lessThanOrEqualTo(deviceCancellationWasteTargetBytes),
    );
    debugPrint(
      'WARP_LONG_CANCEL peerClosed=true '
      'originAcceptedBytes=${cancellationRequest.servedBytes}',
    );
  }
}
