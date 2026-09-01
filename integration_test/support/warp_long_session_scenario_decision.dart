part of 'warp_long_session_scenario.dart';

const _nativeCancellationEvidenceTimeout = Duration(seconds: 15);

extension _WarpLongSessionDecision on _WarpLongSessionDriver {
  Future<void> _captureCancellationDecisionSequence() async {
    final watch = Stopwatch()..start();
    Object? lastError;
    while (watch.elapsed < _nativeCancellationEvidenceTimeout) {
      try {
        final binding = await _cancellationDecisionBinding();
        final history = await graph.evidence.decisions();
        cancellationDecisionSequence = warpExactPendingTransferSequence(
          history.records,
          actionId: binding.actionId,
          postId: binding.postId,
          range: (start: binding.start, end: binding.end),
        );
        return;
      } on StateError catch (error) {
        lastError = error;
      }
      await _tick();
    }
    fail('Exact held request decision binding timed out; $lastError');
  }

  Future<void> _waitForNativeCancellationEvidence() async {
    final watch = Stopwatch()..start();
    while (watch.elapsed < _nativeCancellationEvidenceTimeout) {
      final history = await graph.evidence.decisions();
      final cancellation = warpZeroByteCancellationForSequence(
        history.records,
        cancellationDecisionSequence,
      );
      if (cancellation != null) {
        _reportCancellation(cancellation);
        return;
      }
      await _tick();
    }
    final history = await graph.evidence.decisions();
    fail(
      'Native zero-byte cancellation evidence did not arrive; '
      '${_cancellationEvidence(history.records)}',
    );
  }

  Future<void> _waitForCancellationPeerClose() async {
    try {
      await cancellationGate.peerClosed.timeout(
        _nativeCancellationEvidenceTimeout,
      );
    } on TimeoutException {
      fail(
        'Exact held request did not observe peer close before release; '
        'request=${cancellationRequest.outcome.name}/'
        '${cancellationRequest.servedBytes}',
      );
    }
  }

  void _reportCancellation(WarpDecisionRecord record) {
    debugPrint(
      'WARP_LONG_CANCEL sequence=${record.sequence} '
      'selected=${record.selected?.postId} '
      'executed=${record.executed?.postId} bytes=${record.outcome.bytes}',
    );
  }

  String _cancellationEvidence(Iterable<WarpDecisionRecord> records) {
    final retained = records.toList();
    final summary = retained.reversed
        .take(8)
        .map(
          (record) =>
              '${record.sequence}:${record.outcome.status}:'
              '${record.outcome.bytes}',
        )
        .join('|');
    return 'sequence=$cancellationDecisionSequence '
        '${warpDecisionRetentionEvidence(retained)} '
        'request=${cancellationRequest.outcome.name}/'
        '${cancellationRequest.servedBytes} after='
        '${summary.isEmpty ? 'none' : summary}';
  }
}
