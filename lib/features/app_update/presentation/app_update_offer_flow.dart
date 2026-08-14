part of 'app_update_cubit.dart';

extension AppUpdateOfferFlow on AppUpdateCubit {
  Future<void> _offerAvailable(AppUpdateAvailable available) async {
    final declined = await _readLastDeclinedVersion();
    final preferences = (await _dependencies.settings.load()).updatePreferences;
    if (!preferences.automaticChecks) {
      _emitAvailable(available);
      return;
    }
    if (!_offerPolicy.shouldOffer(
      release: available.release.versionCode,
      lastDeclined: declined,
    )) {
      _emitAvailable(available);
      return;
    }
    _emitState(AppUpdateOfferedState(available.release, available.artifact));
  }

  void _synchronizeOffer(AppUpdatePreferences preferences) {
    final current = state;
    if (!preferences.automaticChecks && current is AppUpdateOfferedState) {
      _emitAvailable(_availableFrom(current)!);
    }
  }

  Future<void> acceptOffer(AndroidVersionCode expected) {
    return _requestOfferAction(expected, AppUpdateOfferAction.accepting);
  }

  Future<void> declineOffer(AndroidVersionCode expected) {
    return _requestOfferAction(expected, AppUpdateOfferAction.declining);
  }

  Future<void> _acceptOffer(AppUpdateAvailable available) async {
    final preferences = (await _dependencies.settings.load()).updatePreferences;
    await _downloadFromIntent(available, preferences);
  }

  Future<void> _declineOffer(
    AppUpdateAvailable available,
    AndroidVersionCode expected,
  ) async {
    if (!await _rememberDecline(expected)) {
      _emitState(
        AppUpdateOfferedState(
          available.release,
          available.artifact,
          message: 'Could not skip this version. Please try again.',
        ),
      );
      return;
    }
    _emitAvailable(available);
  }

  Future<void> _executeOfferAction(
    AndroidVersionCode expected,
    AppUpdateOfferAction action,
  ) async {
    final available = _expectedOffer(expected);
    if (available == null) {
      if (action == AppUpdateOfferAction.declining) {
        await _rememberDecline(expected);
      }
      return;
    }
    _emitPending(available, action);
    try {
      switch (action) {
        case AppUpdateOfferAction.accepting:
          await _acceptOffer(available);
        case AppUpdateOfferAction.declining:
          await _declineOffer(available, expected);
      }
    } on AppFailure catch (failure) {
      _emitOfferFailure(available, failure.message);
    } on Object catch (error, stackTrace) {
      _emitOfferFailure(available, _unexpected(error, stackTrace));
    }
  }

  void _emitOfferFailure(AppUpdateAvailable available, String message) {
    _emitState(
      AppUpdateOfferedState(
        available.release,
        available.artifact,
        message: message,
      ),
    );
  }

  Future<void> _requestOfferAction(
    AndroidVersionCode expected,
    AppUpdateOfferAction action,
  ) {
    final queued = _offerIntent;
    if (queued != null) return queued.completion.future;
    final available = _expectedOffer(expected, readyOnly: true);
    if (available == null) return Future<void>.value();
    if (!_operationActive) {
      return _run(() => _executeOfferAction(expected, action));
    }
    final intent = _PendingOfferIntent(expected, action);
    _offerIntent = intent;
    _emitPending(available, action);
    return intent.completion.future;
  }

  Future<void> _drainOfferIntent() async {
    final intent = _offerIntent;
    if (intent == null || _operationActive) return;
    _offerIntent = null;
    if (!isClosed) {
      await _run(() => _executeOfferAction(intent.expected, intent.action));
    }
    intent.completion.complete();
  }

  void _emitPending(AppUpdateAvailable available, AppUpdateOfferAction action) {
    _emitState(
      AppUpdateOfferedState(
        available.release,
        available.artifact,
        pendingAction: action,
      ),
    );
  }

  AppUpdateAvailable? _expectedOffer(
    AndroidVersionCode expected, {
    bool readyOnly = false,
  }) {
    final current = state;
    if (current is! AppUpdateOfferedState ||
        current.release.versionCode != expected ||
        (readyOnly && current.pendingAction != null)) {
      return null;
    }
    return AppUpdateAvailable(
      release: current.release,
      artifact: current.artifact,
    );
  }

  Future<AndroidVersionCode?> _readLastDeclinedVersion() async {
    if (!_offerHistoryLoaded) {
      try {
        _lastDeclinedVersion = await _dependencies.offerHistory
            .readLastDeclinedVersion();
      } on Object catch (error, stackTrace) {
        logBoundaryFailure(
          source: 'ghostr.update.offer-history',
          message: 'Could not read declined update history.',
          error: error,
          stackTrace: stackTrace,
        );
      }
      _offerHistoryLoaded = true;
    }
    return _lastDeclinedVersion;
  }

  Future<bool> _persistDecline(AndroidVersionCode version) async {
    try {
      await _dependencies.offerHistory.recordDeclinedVersion(version);
      return true;
    } on Object catch (error, stackTrace) {
      logBoundaryFailure(
        source: 'ghostr.update.offer-history',
        message: 'Could not persist a declined update offer.',
        error: error,
        stackTrace: stackTrace,
      );
      return false;
    }
  }

  Future<bool> _rememberDecline(AndroidVersionCode version) async {
    if (!await _persistDecline(version)) return false;
    _lastDeclinedVersion = version;
    _offerHistoryLoaded = true;
    return true;
  }
}

final class _PendingOfferIntent {
  _PendingOfferIntent(this.expected, this.action);

  final AndroidVersionCode expected;
  final AppUpdateOfferAction action;
  final Completer<void> completion = Completer<void>();
}
