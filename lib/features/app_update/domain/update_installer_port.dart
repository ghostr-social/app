import 'package:ghostr/features/app_update/domain/verified_update_package.dart';

enum UpdateInstallPermission { granted, required, unsupported }

enum UpdateInstallMode { automaticWhenPermitted, confirmationRequired }

enum UpdateInstallStatus { pending, awaitingUserAction, succeeded, failed }

final class UpdateInstallRequest {
  const UpdateInstallRequest({required this.package, required this.mode});

  final VerifiedUpdatePackage package;
  final UpdateInstallMode mode;
}

final class UpdateInstallSession {
  factory UpdateInstallSession(int id) {
    if (id < 0) throw ArgumentError.value(id, 'id', 'Must not be negative.');
    return UpdateInstallSession._(id);
  }

  const UpdateInstallSession._(this.id);

  final int id;
}

abstract interface class UpdateInstallerPort {
  Future<UpdateInstallPermission> readPermission();

  Future<void> openPermissionSettings();

  Future<UpdateInstallSession> submit(UpdateInstallRequest request);

  Future<UpdateInstallSession> replace(
    UpdateInstallSession previous,
    UpdateInstallRequest request,
  );

  Future<UpdateInstallStatus> readStatus(UpdateInstallSession session);
}
