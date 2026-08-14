import 'package:flutter/material.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

part 'app_update_status_view.dart';

final class AppUpdateStatusActions {
  const AppUpdateStatusActions({
    required this.onDownload,
    required this.onInstall,
    required this.onOpenPermissionSettings,
    required this.onRetryPermission,
    required this.onRetryConfirmation,
    required this.onRefreshStatus,
  });

  final VoidCallback onDownload;
  final VoidCallback onInstall;
  final VoidCallback onOpenPermissionSettings;
  final VoidCallback onRetryPermission;
  final VoidCallback onRetryConfirmation;
  final VoidCallback onRefreshStatus;
}

class AppUpdateStatusPanel extends StatelessWidget {
  const AppUpdateStatusPanel({
    required this.state,
    required this.actions,
    super.key,
  });

  final AppUpdateState state;
  final AppUpdateStatusActions actions;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(AppSpacing.md),
        child: _passiveStatus(state),
      ),
    );
  }

  Widget _passiveStatus(AppUpdateState value) {
    if (value is AppUpdateIdleState) {
      return const _StatusView(
        icon: Icons.update,
        message: 'No update check has run yet.',
      );
    }
    if (value is AppUpdateCheckingState) {
      return const _StatusView(
        icon: Icons.sync,
        message: 'Checking for updates…',
        progressLabel: 'Checking for updates',
      );
    }
    if (value is AppUpdateCurrentState) {
      return const _StatusView(
        icon: Icons.check_circle_outline,
        message: 'Ghostr is up to date.',
      );
    }
    return _downloadStatus(value);
  }

  Widget _downloadStatus(AppUpdateState value) {
    if (value is AppUpdateAvailableState) {
      return _availableStatus(value.release.versionName);
    }
    if (value is AppUpdateOfferedState) {
      return _availableStatus(
        value.release.versionName,
        detail: value.message,
        pendingAction: value.pendingAction,
      );
    }
    if (value is AppUpdateWaitingForWifiState) {
      return _StatusView(
        icon: Icons.wifi,
        message: 'Waiting for Wi-Fi.',
        detail: 'Ghostr ${value.release.versionName} will download on Wi-Fi.',
      );
    }
    if (value is AppUpdateDownloadingState) {
      return _StatusView(
        icon: Icons.downloading,
        message: 'Downloading Ghostr ${value.release.versionName}…',
        detail: '${(value.fraction * 100).round()}%',
        progressLabel: 'Downloading update',
        progress: value.fraction,
      );
    }
    return _installStatus(value);
  }

  Widget _availableStatus(
    String versionName, {
    String? detail,
    AppUpdateOfferAction? pendingAction,
  }) {
    return _StatusView(
      icon: Icons.system_update_alt,
      message: 'Ghostr $versionName is available.',
      detail: detail,
      detailIsError: detail != null,
      progressLabel: _offerProgressLabel(pendingAction),
      primary: _PanelAction(
        'Download update',
        pendingAction == null ? actions.onDownload : null,
      ),
    );
  }

  Widget _installStatus(AppUpdateState value) {
    if (value is AppUpdateReadyState) {
      return _StatusView(
        icon: Icons.install_mobile,
        message: 'The update is ready to install.',
        primary: _PanelAction('Install update', actions.onInstall),
      );
    }
    if (value is AppUpdatePermissionRequiredState) {
      return _StatusView(
        icon: Icons.security,
        message: 'Allow Ghostr to install updates.',
        primary: _PanelAction(
          'Allow updates',
          actions.onOpenPermissionSettings,
        ),
        secondary: _PanelAction(
          'Retry installation',
          actions.onRetryPermission,
        ),
      );
    }
    return _terminalStatus(value);
  }

  Widget _terminalStatus(AppUpdateState value) {
    if (value is AppUpdateInstallingState) {
      return _installingStatus(value);
    }
    if (value is AppUpdateFailureState) {
      return _StatusView(
        icon: Icons.error_outline,
        message: 'Update failed',
        detail: value.message,
        detailIsError: true,
      );
    }
    if (value is AppUpdateUnsupportedState) {
      return _StatusView(
        icon: Icons.mobile_off,
        message: 'Updates unavailable',
        detail: value.message,
      );
    }
    throw StateError('Unhandled app update state: $value');
  }

  Widget _installingStatus(AppUpdateInstallingState value) {
    final awaiting = value.status == UpdateInstallStatus.awaitingUserAction;
    return _StatusView(
      icon: Icons.install_mobile,
      message: _installMessage(value.status),
      progressLabel: 'Installing update',
      primary: awaiting
          ? _PanelAction('Open Android installer', actions.onRetryConfirmation)
          : null,
      outlined: _PanelAction('Refresh install status', actions.onRefreshStatus),
    );
  }
}

String _installMessage(UpdateInstallStatus status) => switch (status) {
  UpdateInstallStatus.pending => 'Preparing the Android installer…',
  UpdateInstallStatus.awaitingUserAction => 'Confirm the update in Android.',
  UpdateInstallStatus.succeeded =>
    'Android reported success. Verifying the installed version…',
  UpdateInstallStatus.failed => 'Android could not install the update.',
};

String? _offerProgressLabel(AppUpdateOfferAction? action) => switch (action) {
  AppUpdateOfferAction.accepting => 'Starting update',
  AppUpdateOfferAction.declining => 'Saving skipped version',
  null => null,
};
