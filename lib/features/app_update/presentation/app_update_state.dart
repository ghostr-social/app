import 'package:ghostr/features/app_update/domain/release_artifact.dart';
import 'package:ghostr/features/app_update/domain/stable_release.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/features/app_update/domain/verified_update_package.dart';

sealed class AppUpdateState {
  const AppUpdateState();
}

final class AppUpdateIdleState extends AppUpdateState {
  const AppUpdateIdleState();
}

final class AppUpdateCheckingState extends AppUpdateState {
  const AppUpdateCheckingState();
}

final class AppUpdateCurrentState extends AppUpdateState {
  const AppUpdateCurrentState();
}

final class AppUpdateAvailableState extends AppUpdateState {
  const AppUpdateAvailableState(this.release, this.artifact);

  final StableRelease release;
  final ReleaseArtifact artifact;
}

enum AppUpdateOfferAction { accepting, declining }

final class AppUpdateOfferedState extends AppUpdateState {
  const AppUpdateOfferedState(
    this.release,
    this.artifact, {
    this.message,
    this.pendingAction,
  });

  final StableRelease release;
  final ReleaseArtifact artifact;
  final String? message;
  final AppUpdateOfferAction? pendingAction;
}

final class AppUpdateWaitingForWifiState extends AppUpdateState {
  const AppUpdateWaitingForWifiState(this.release, this.artifact);

  final StableRelease release;
  final ReleaseArtifact artifact;
}

final class AppUpdateDownloadingState extends AppUpdateState {
  const AppUpdateDownloadingState({
    required this.release,
    required this.artifact,
    required this.bytes,
    required this.totalBytes,
  });

  final StableRelease release;
  final ReleaseArtifact artifact;
  final int bytes;
  final int totalBytes;

  double get fraction => totalBytes == 0 ? 0 : bytes / totalBytes;
}

final class AppUpdateReadyState extends AppUpdateState {
  const AppUpdateReadyState(this.package);

  final VerifiedUpdatePackage package;
}

final class AppUpdatePermissionRequiredState extends AppUpdateState {
  const AppUpdatePermissionRequiredState(this.package, this.mode);

  final VerifiedUpdatePackage package;
  final UpdateInstallMode mode;
}

final class AppUpdateInstallingState extends AppUpdateState {
  const AppUpdateInstallingState({
    required this.package,
    required this.session,
    required this.status,
  });

  final VerifiedUpdatePackage package;
  final UpdateInstallSession session;
  final UpdateInstallStatus status;
}

final class AppUpdateFailureState extends AppUpdateState {
  const AppUpdateFailureState(this.message);

  final String message;
}

final class AppUpdateUnsupportedState extends AppUpdateState {
  const AppUpdateUnsupportedState(this.message);

  final String message;
}
