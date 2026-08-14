part of 'app_update_cubit.dart';

extension AppUpdateDownloadFlow on AppUpdateCubit {
  AppUpdateAvailable? _availableFrom(AppUpdateState current) {
    return switch (current) {
      AppUpdateAvailableState(:final release, :final artifact) =>
        AppUpdateAvailable(release: release, artifact: artifact),
      AppUpdateOfferedState(:final release, :final artifact) =>
        AppUpdateAvailable(release: release, artifact: artifact),
      AppUpdateWaitingForWifiState(:final release, :final artifact) =>
        AppUpdateAvailable(release: release, artifact: artifact),
      _ => null,
    };
  }

  Future<void> _downloadWhenConnected(
    AppUpdateAvailable available,
    AppUpdatePreferences preferences,
  ) async {
    final connection = await _dependencies.network.readConnection();
    if (_requiresWifi(preferences, connection)) {
      _emitWaitingForWifi(available);
      return;
    }
    if (connection == NetworkConnection.offline) {
      throw const AppFailure('Connect to the internet to download the update.');
    }
    await _download(available);
  }

  Future<void> _downloadFromIntent(
    AppUpdateAvailable available,
    AppUpdatePreferences preferences,
  ) async {
    final connection = await _dependencies.network.readConnection();
    if (_requiresWifi(preferences, connection)) {
      _emitWaitingForWifi(available);
    } else if (connection == NetworkConnection.offline) {
      throw const AppFailure('Connect to the internet to download the update.');
    } else {
      await _download(available);
    }
  }

  bool _requiresWifi(
    AppUpdatePreferences preferences,
    NetworkConnection connection,
  ) {
    return preferences.downloadPolicy == UpdateDownloadPolicy.wifiOnly &&
        connection != NetworkConnection.wifi;
  }

  void _emitWaitingForWifi(AppUpdateAvailable available) {
    _emitState(
      AppUpdateWaitingForWifiState(available.release, available.artifact),
    );
  }

  Future<void> _download(AppUpdateAvailable available) async {
    _emitDownloadProgress(available, 0, available.artifact.sizeBytes);
    await for (final event in _dependencies.downloader.download(
      available.release,
      available.artifact,
    )) {
      await _acceptDownloadEvent(available, event);
    }
  }

  Future<void> _acceptDownloadEvent(
    AppUpdateAvailable available,
    UpdateDownloadEvent event,
  ) async {
    switch (event) {
      case UpdateDownloadProgress(:final bytes, :final totalBytes):
        _emitDownloadProgress(available, bytes, totalBytes);
      case UpdateDownloadCompleted(:final package):
        await _acceptPackage(package);
    }
  }

  void _emitDownloadProgress(
    AppUpdateAvailable available,
    int bytes,
    int totalBytes,
  ) {
    _emitState(
      AppUpdateDownloadingState(
        release: available.release,
        artifact: available.artifact,
        bytes: bytes,
        totalBytes: totalBytes,
      ),
    );
  }

  Future<void> _acceptPackage(VerifiedUpdatePackage package) async {
    final settings = await _dependencies.settings.load();
    final preferences = settings.updatePreferences;
    if (!preferences.automaticInstall) {
      _emitState(AppUpdateReadyState(package));
      return;
    }
    await _prepareInstall(package, UpdateInstallMode.automaticWhenPermitted);
  }
}
