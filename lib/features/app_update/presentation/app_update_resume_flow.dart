part of 'app_update_cubit.dart';

extension AppUpdateResumeFlow on AppUpdateCubit {
  Future<bool> _continueOnResume(AppUpdatePreferences preferences) async {
    final current = state;
    if (current is AppUpdateWaitingForWifiState) {
      await _resumeWaitingDownload(current, preferences);
      return true;
    }
    if (current is AppUpdatePermissionRequiredState) {
      await _resumePermission(current, preferences);
      return true;
    }
    if (current is AppUpdateInstallingState) {
      await _readInstallStatus(current.package, current.session);
      return true;
    }
    return false;
  }

  Future<void> _resumeWaitingDownload(
    AppUpdateWaitingForWifiState current,
    AppUpdatePreferences preferences,
  ) async {
    final available = AppUpdateAvailable(
      release: current.release,
      artifact: current.artifact,
    );
    if (preferences.downloadPolicy == UpdateDownloadPolicy.manual) {
      _emitAvailable(available);
      return;
    }
    await _downloadWhenConnected(available, preferences);
  }

  Future<void> _resumePermission(
    AppUpdatePermissionRequiredState current,
    AppUpdatePreferences preferences,
  ) async {
    final automatic = current.mode == UpdateInstallMode.automaticWhenPermitted;
    if (automatic && !preferences.automaticInstall) {
      _emitState(AppUpdateReadyState(current.package));
      return;
    }
    await _prepareInstall(current.package, current.mode);
  }
}
