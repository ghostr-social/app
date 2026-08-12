import 'dart:async';

import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';
import 'package:ghostr/features/app_update/domain/installed_app.dart';
import 'package:ghostr/features/app_update/domain/installed_app_port.dart';
import 'package:ghostr/features/app_update/domain/network_connection_port.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/platform/app_update/android_app_update_platform.dart';

final class AndroidAppUpdateAdapter
    implements InstalledAppPort, NetworkConnectionPort, UpdateInstallerPort {
  AndroidAppUpdateAdapter(this._platform) {
    _statusSubscription = _platform.statuses.listen(_recordStatus);
  }

  final AndroidAppUpdatePlatform _platform;
  final Map<int, UpdateInstallStatus> _statuses = {};
  late final StreamSubscription<AndroidInstallStatus> _statusSubscription;

  @override
  Future<InstalledApp> readInstalledApp() => _guard('identity', () async {
    final app = await _platform.getInstalledApp();
    return InstalledApp(
      packageName: app.packageName,
      versionName: app.versionName,
      versionCode: AndroidVersionCode(app.versionCode),
      supportedAbis: app.supportedAbis
          .map(AndroidAbi.tryParse)
          .whereType<AndroidAbi>()
          .toList(),
    );
  });

  @override
  Future<NetworkConnection> readConnection() => _guard('network', () async {
    final access = await _platform.getNetworkAccess();
    return switch (access) {
      AndroidNetworkAccess.none => NetworkConnection.offline,
      AndroidNetworkAccess.wifi => NetworkConnection.wifi,
      AndroidNetworkAccess.other => NetworkConnection.other,
    };
  });

  @override
  Future<UpdateInstallPermission> readPermission() {
    return _guard('permission', () async {
      final allowed = await _platform.canRequestInstalls();
      return allowed
          ? UpdateInstallPermission.granted
          : UpdateInstallPermission.required;
    });
  }

  @override
  Future<void> openPermissionSettings() {
    return _guard(
      'permission-settings',
      _platform.openInstallPermissionSettings,
    );
  }

  @override
  Future<UpdateInstallSession> submit(UpdateInstallRequest request) {
    return _guard('install', () async {
      final sessionId = await _platform.install(_installRequest(request));
      return UpdateInstallSession(sessionId);
    });
  }

  @override
  Future<UpdateInstallSession> replace(
    UpdateInstallSession previous,
    UpdateInstallRequest request,
  ) {
    return _guard('replace-install', () async {
      final sessionId = await _platform.replaceInstall(
        previous.id,
        _installRequest(request),
      );
      return UpdateInstallSession(sessionId);
    });
  }

  @override
  Future<UpdateInstallStatus> readStatus(UpdateInstallSession session) async {
    final current = _statuses[session.id];
    if (current != null) return current;
    final restored = await _readRestoredStatus(session.id);
    if (restored != null) return restored;
    final received = _statuses[session.id];
    if (received != null) return received;
    return UpdateInstallStatus.pending;
  }

  Future<void> dispose() async {
    await _statusSubscription.cancel();
    _platform.dispose();
  }

  void _recordStatus(AndroidInstallStatus status) {
    final mapped = _installStatus(status.state);
    _statuses[status.sessionId] = mapped;
  }

  Future<UpdateInstallStatus?> _readRestoredStatus(int sessionId) async {
    try {
      final status = await _platform.readInstallStatus(sessionId);
      if (status == null) return null;
      _recordStatus(status);
      return _installStatus(status.state);
    } on Object catch (error, stackTrace) {
      logBoundaryFailure(
        source: 'ghostr.update.android.status',
        message: 'Android could not restore the update status.',
        error: error,
        stackTrace: stackTrace,
      );
      return null;
    }
  }
}

AndroidInstallMode _installMode(UpdateInstallMode mode) => switch (mode) {
  UpdateInstallMode.automaticWhenPermitted => AndroidInstallMode.automatic,
  UpdateInstallMode.confirmationRequired => AndroidInstallMode.userConfirmed,
};

AndroidInstallRequest _installRequest(UpdateInstallRequest request) {
  return AndroidInstallRequest(
    path: request.package.path,
    expectedVersionCode: request.package.versionCode.value,
    mode: _installMode(request.mode),
  );
}

UpdateInstallStatus _installStatus(AndroidInstallState state) =>
    switch (state) {
      AndroidInstallState.pendingUserAction =>
        UpdateInstallStatus.awaitingUserAction,
      AndroidInstallState.succeeded => UpdateInstallStatus.succeeded,
      AndroidInstallState.failed => UpdateInstallStatus.failed,
    };

Future<T> _guard<T>(String operation, Future<T> Function() action) async {
  try {
    return await action();
  } on Object catch (error, stackTrace) {
    throw translatedBoundaryFailure(
      source: 'ghostr.update.android.$operation',
      message: 'Android app updates are unavailable.',
      error: error,
      stackTrace: stackTrace,
    );
  }
}
