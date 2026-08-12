part of 'app_update_cubit.dart';

extension AppUpdateCheckFlow on AppUpdateCubit {
  Future<void> _check(AppUpdatePreferences preferences) async {
    _emitState(const AppUpdateCheckingState());
    final installed = await _dependencies.installedApp.readInstalledApp();
    final release = await _dependencies.catalog.fetchStableRelease();
    final availability = _policy.evaluate(
      installed: installed,
      release: release,
    );
    await _acceptAvailability(availability, preferences);
    _lastCheckAt = _clock();
  }

  Future<void> _acceptAvailability(
    AppUpdateAvailability availability,
    AppUpdatePreferences preferences,
  ) async {
    switch (availability) {
      case AppUpdateCurrent():
        _emitState(const AppUpdateCurrentState());
      case AppUpdateUnsupported():
        _emitUnsupportedDevice();
      case AppUpdateAvailable():
        await _acceptAvailable(availability, preferences);
    }
  }

  Future<void> _acceptAvailable(
    AppUpdateAvailable available,
    AppUpdatePreferences preferences,
  ) async {
    if (preferences.downloadPolicy == UpdateDownloadPolicy.manual) {
      _emitAvailable(available);
      return;
    }
    await _downloadWhenConnected(available, preferences);
  }

  void _emitAvailable(AppUpdateAvailable available) {
    _emitState(AppUpdateAvailableState(available.release, available.artifact));
  }

  void _emitUnsupportedDevice() {
    _emitState(
      const AppUpdateUnsupportedState(
        'This update is not available for this device.',
      ),
    );
  }
}
