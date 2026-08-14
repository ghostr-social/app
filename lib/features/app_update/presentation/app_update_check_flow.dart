part of 'app_update_cubit.dart';

enum _AppUpdateCheckTrigger { automatic, manual }

extension AppUpdateCheckFlow on AppUpdateCubit {
  Future<void> _check(_AppUpdateCheckTrigger trigger) async {
    _scheduleAutomaticCheck(AppUpdateCubit.foregroundPollInterval);
    final retainedOffer = _retainedOffer(trigger);
    if (retainedOffer != null) {
      await _refreshOfferSafely(retainedOffer);
      return;
    }
    _emitState(const AppUpdateCheckingState());
    final availability = await _readAvailability();
    _scheduleAutomaticCheck(AppUpdateCubit.foregroundCheckInterval);
    await _acceptAvailability(availability, trigger);
  }

  Future<AppUpdateAvailability> _readAvailability() async {
    final installed = await _dependencies.installedApp.readInstalledApp();
    final release = await _dependencies.catalog.fetchStableRelease();
    return _policy.evaluate(installed: installed, release: release);
  }

  Future<void> _refreshOfferSafely(AppUpdateOfferedState retained) async {
    try {
      final availability = await _readAvailability();
      _scheduleAutomaticCheck(AppUpdateCubit.foregroundCheckInterval);
      await _refreshOffer(retained, availability);
    } on Object catch (error, stackTrace) {
      logBoundaryFailure(
        source: 'ghostr.update.refresh',
        message: 'Could not refresh an outstanding update offer.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }

  AppUpdateOfferedState? _retainedOffer(_AppUpdateCheckTrigger trigger) {
    final current = state;
    return trigger == _AppUpdateCheckTrigger.automatic &&
            current is AppUpdateOfferedState
        ? current
        : null;
  }

  Future<void> _refreshOffer(
    AppUpdateOfferedState retained,
    AppUpdateAvailability availability,
  ) async {
    switch (availability) {
      case AppUpdateCurrent():
        _emitState(const AppUpdateCurrentState());
      case AppUpdateAvailable()
          when availability.release.versionCode.compareTo(
                retained.release.versionCode,
              ) >
              0:
        await _offerAvailable(availability);
      case AppUpdateAvailable() || AppUpdateUnsupported():
        return;
    }
  }

  Future<void> _acceptAvailability(
    AppUpdateAvailability availability,
    _AppUpdateCheckTrigger trigger,
  ) async {
    switch (availability) {
      case AppUpdateCurrent():
        _emitState(const AppUpdateCurrentState());
      case AppUpdateUnsupported():
        _emitUnsupported(availability.reason);
      case AppUpdateAvailable():
        await _acceptAvailable(availability, trigger);
    }
  }

  Future<void> _acceptAvailable(
    AppUpdateAvailable available,
    _AppUpdateCheckTrigger trigger,
  ) async {
    if (trigger == _AppUpdateCheckTrigger.manual) {
      _emitAvailable(available);
    } else {
      await _offerAvailable(available);
    }
  }

  void _emitAvailable(AppUpdateAvailable available) {
    _emitState(AppUpdateAvailableState(available.release, available.artifact));
  }

  void _emitUnsupported(AppUpdateUnsupportedReason reason) {
    final message = switch (reason) {
      AppUpdateUnsupportedReason.noCompatibleArtifact =>
        'This update is not available for this device.',
      AppUpdateUnsupportedReason.nonIncreasingReleaseCode =>
        'A newer version cannot replace this installed build.',
    };
    _emitState(AppUpdateUnsupportedState(message));
  }
}
