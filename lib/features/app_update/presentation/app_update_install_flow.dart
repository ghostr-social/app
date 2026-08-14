part of 'app_update_cubit.dart';

extension AppUpdateInstallFlow on AppUpdateCubit {
  Future<void> _prepareInstall(
    VerifiedUpdatePackage package,
    UpdateInstallMode mode,
  ) async {
    final permission = await _dependencies.installer.readPermission();
    switch (permission) {
      case UpdateInstallPermission.granted:
        await _submitInstall(package, mode);
      case UpdateInstallPermission.required:
        _emitState(AppUpdatePermissionRequiredState(package, mode));
      case UpdateInstallPermission.unsupported:
        _emitState(
          const AppUpdateUnsupportedState(
            'Automatic installation is not supported on this device.',
          ),
        );
    }
  }

  Future<void> _submitInstall(
    VerifiedUpdatePackage package,
    UpdateInstallMode mode,
  ) async {
    final request = UpdateInstallRequest(package: package, mode: mode);
    final session = await _dependencies.installer.submit(request);
    _emitState(
      AppUpdateInstallingState(
        package: package,
        session: session,
        status: UpdateInstallStatus.pending,
      ),
    );
    await _readInstallStatus(package, session);
  }

  Future<void> _readInstallStatus(
    VerifiedUpdatePackage package,
    UpdateInstallSession session,
  ) async {
    final status = await _dependencies.installer.readStatus(session);
    switch (status) {
      case UpdateInstallStatus.pending:
        return;
      case UpdateInstallStatus.awaitingUserAction:
        _emitInstalling(package, session, status);
      case UpdateInstallStatus.succeeded:
        await _verifyInstalled(package, session);
      case UpdateInstallStatus.failed:
        _emitState(
          const AppUpdateFailureState('Android could not install the update.'),
        );
    }
  }

  Future<void> _verifyInstalled(
    VerifiedUpdatePackage package,
    UpdateInstallSession session,
  ) async {
    final installed = await _dependencies.installedApp.readInstalledApp();
    if (installed.versionCode.compareTo(package.versionCode) >= 0) {
      _emitState(const AppUpdateCurrentState());
    } else {
      _emitInstalling(package, session, UpdateInstallStatus.succeeded);
    }
  }

  Future<void> _replaceInstall(
    VerifiedUpdatePackage package,
    UpdateInstallSession previous,
  ) async {
    final request = UpdateInstallRequest(
      package: package,
      mode: UpdateInstallMode.confirmationRequired,
    );
    final session = await _dependencies.installer.replace(previous, request);
    _emitInstalling(package, session, UpdateInstallStatus.pending);
    await _readInstallStatus(package, session);
  }

  void _emitInstalling(
    VerifiedUpdatePackage package,
    UpdateInstallSession session,
    UpdateInstallStatus status,
  ) {
    _emitState(
      AppUpdateInstallingState(
        package: package,
        session: session,
        status: status,
      ),
    );
  }
}
