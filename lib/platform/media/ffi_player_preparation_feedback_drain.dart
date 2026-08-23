part of 'ffi_player_preparation_feedback_port.dart';

const _preparationRetryDelays = <Duration>[
  Duration(milliseconds: 10),
  Duration(milliseconds: 20),
  Duration(milliseconds: 40),
  Duration(milliseconds: 80),
  Duration(milliseconds: 160),
  Duration(milliseconds: 320),
  Duration(milliseconds: 500),
];

extension _FfiPlayerPreparationFeedbackDrain
    on FfiPlayerPreparationFeedbackPort {
  Future<void> _drain(BigInt attempt) async {
    var retryIndex = 0;
    while (!_closed && _pending[attempt]?.isNotEmpty == true) {
      final report = _pending[attempt]!.first;
      if (report.state == FfiPlayerPreparationState.initializing) {
        _dispatchedAttempts.add(attempt);
      }
      _noAdmissionAttempts.remove(attempt);
      final disposition = await _deliver(report);
      if (_closed) break;
      if (disposition == null || !_consume(report, disposition)) {
        retryIndex = await _backoff(attempt, retryIndex);
      } else {
        retryIndex = 0;
      }
    }
    _draining.remove(attempt);
    assert(_noAdmissionAttempts.every(_attempts.containsKey));
    assert(_retryWakes.keys.every(_attempts.containsKey));
  }

  Future<FfiPlayerPreparationDisposition?> _deliver(
    FfiPlayerPreparationReport report,
  ) async {
    try {
      return await _reportPreparation(input: report);
    } on Object catch (error, stackTrace) {
      developer.log(
        'Player preparation evidence did not reach WARP.',
        name: 'ghostr.video.preparation',
        error: error,
        stackTrace: stackTrace,
      );
      return null;
    }
  }

  bool _consume(
    FfiPlayerPreparationReport report,
    FfiPlayerPreparationDisposition disposition,
  ) {
    if (_acknowledged(disposition)) {
      _acknowledge(report);
      return true;
    }
    if (_discarded(disposition)) {
      _logDiscard(report, disposition);
      _discardAttempt(report.attemptGeneration);
      return true;
    }
    return _consumeControl(report, disposition);
  }

  bool _consumeControl(
    FfiPlayerPreparationReport report,
    FfiPlayerPreparationDisposition disposition,
  ) {
    if (disposition == FfiPlayerPreparationDisposition.missingInitial) {
      _restorePrefix(report);
      return true;
    }
    if (disposition == FfiPlayerPreparationDisposition.closed) {
      _close();
      return true;
    }
    return _consumeRetry(report, disposition);
  }

  bool _consumeRetry(
    FfiPlayerPreparationReport report,
    FfiPlayerPreparationDisposition disposition,
  ) {
    if (disposition == FfiPlayerPreparationDisposition.saturated) {
      if (_history[report.attemptGeneration]?.isNotEmpty == true) return false;
      return _retryWithoutAdmission(report);
    }
    if (disposition == FfiPlayerPreparationDisposition.notAdmitted) {
      return _retryWithoutAdmission(report);
    }
    return false;
  }

  bool _retryWithoutAdmission(FfiPlayerPreparationReport report) {
    final attempt = report.attemptGeneration;
    final initial =
        report.sequence == BigInt.one &&
        report.state == FfiPlayerPreparationState.initializing;
    if (!initial || _history[attempt]?.isNotEmpty == true) return false;
    if (_attempts[attempt]?._terminal == true) {
      _discardAttempt(attempt);
      return true;
    }
    _noAdmissionAttempts.add(attempt);
    return false;
  }

  void _acknowledge(FfiPlayerPreparationReport report) {
    final queue = _pending[report.attemptGeneration]!;
    queue.removeFirst();
    if (queue.isEmpty) _pending.remove(report.attemptGeneration);
    final history = _history.putIfAbsent(report.attemptGeneration, () => []);
    if (!history.contains(report)) history.add(report);
    if (!_isTerminal(report.state)) return;
    _dispatchedAttempts.remove(report.attemptGeneration);
    _attempts.remove(report.attemptGeneration);
    _history.remove(report.attemptGeneration);
    _noAdmissionAttempts.remove(report.attemptGeneration);
    _wakeRetry(report.attemptGeneration);
  }

  void _restorePrefix(FfiPlayerPreparationReport report) {
    final history = _history[report.attemptGeneration] ?? const [];
    final prefix = history
        .where((item) => item.sequence < report.sequence)
        .toList(growable: false);
    if (prefix.isEmpty) {
      _discardAttempt(report.attemptGeneration);
      return;
    }
    final queue = _pending[report.attemptGeneration]!;
    final queued = queue.toList(growable: false);
    queue
      ..clear()
      ..addAll(prefix)
      ..addAll(queued);
  }

  void _discardAttempt(BigInt attempt) {
    _pending.remove(attempt);
    _dispatchedAttempts.remove(attempt);
    _attempts.remove(attempt)?._discard();
    _history.remove(attempt);
    _noAdmissionAttempts.remove(attempt);
    _wakeRetry(attempt);
  }

  void _close() {
    if (_closed) return;
    _closed = true;
    for (final attempt in _attempts.values) {
      attempt._discard();
    }
    _pending.clear();
    _dispatchedAttempts.clear();
    _attempts.clear();
    _history.clear();
    _noAdmissionAttempts.clear();
    for (final wake in _retryWakes.values) {
      if (!wake.isCompleted) wake.complete();
    }
    _retryWakes.clear();
    _draining.clear();
  }

  void _logDiscard(
    FfiPlayerPreparationReport report,
    FfiPlayerPreparationDisposition disposition,
  ) {
    developer.log(
      'Player preparation ${disposition.name}: ${report.state.name}; '
      'capability=${report.playerCapabilityGeneration}, '
      'epoch=${report.clientEpoch}, attempt=${report.attemptGeneration}, '
      'sequence=${report.sequence}.',
      name: 'ghostr.video.preparation',
    );
  }

  Future<int> _backoff(BigInt attempt, int index) async {
    final delay = _preparationRetryDelays[index];
    final next = index < _preparationRetryDelays.length - 1 ? index + 1 : index;
    final wake = Completer<void>();
    _retryWakes[attempt] = wake;
    await Future.any<void>([Future<void>.delayed(delay), wake.future]);
    if (identical(_retryWakes[attempt], wake)) _retryWakes.remove(attempt);
    return next;
  }

  void _wakeRetry(BigInt attempt) {
    final wake = _retryWakes.remove(attempt);
    if (wake != null && !wake.isCompleted) wake.complete();
  }
}
