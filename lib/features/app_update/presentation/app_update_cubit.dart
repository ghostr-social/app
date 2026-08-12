import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/core/time/clock.dart';
import 'package:ghostr/features/app_update/domain/network_connection_port.dart';
import 'package:ghostr/features/app_update/domain/update_availability.dart';
import 'package:ghostr/features/app_update/domain/update_availability_policy.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/features/app_update/domain/update_package_downloader.dart';
import 'package:ghostr/features/app_update/domain/verified_update_package.dart';
import 'package:ghostr/features/app_update/presentation/app_update_dependencies.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

export 'app_update_dependencies.dart';
export 'app_update_state.dart';

part 'app_update_check_flow.dart';
part 'app_update_download_flow.dart';
part 'app_update_install_flow.dart';
part 'app_update_resume_flow.dart';

final class AppUpdateCubit extends DisposalSafeCubit<AppUpdateState> {
  AppUpdateCubit(
    this._dependencies, {
    UpdateAvailabilityPolicy policy = const UpdateAvailabilityPolicy(),
    Clock clock = systemClock,
  }) : _policy = policy,
       _clock = clock,
       super(const AppUpdateIdleState());

  static const foregroundCheckInterval = Duration(hours: 6);

  final AppUpdateDependencies _dependencies;
  final UpdateAvailabilityPolicy _policy;
  final Clock _clock;
  bool _operationActive = false;
  DateTime? _lastCheckAt;

  Future<void> start() => _run(() async {
    final preferences = (await _dependencies.settings.load()).updatePreferences;
    if (preferences.automaticChecks) await _check(preferences);
  });

  Future<void> checkNow() => _run(() async {
    final preferences = (await _dependencies.settings.load()).updatePreferences;
    await _check(preferences);
  });

  Future<void> onAppResumed() => _run(() async {
    final preferences = (await _dependencies.settings.load()).updatePreferences;
    if (await _continueOnResume(preferences)) return;
    if (!preferences.automaticChecks || !_foregroundCheckDue()) return;
    await _check(preferences);
  });

  Future<void> downloadAvailable() => _run(() async {
    final available = _availableFrom(state);
    if (available == null) return;
    final preferences = (await _dependencies.settings.load()).updatePreferences;
    await _downloadFromIntent(available, preferences);
  });

  Future<void> retryDownload() => downloadAvailable();

  Future<void> installReady() => _run(() async {
    final current = state;
    if (current is! AppUpdateReadyState) return;
    await _prepareInstall(
      current.package,
      UpdateInstallMode.confirmationRequired,
    );
  });

  Future<void> retryInstall() => _run(() async {
    final current = state;
    if (current is! AppUpdatePermissionRequiredState) return;
    await _prepareInstall(current.package, current.mode);
  });

  Future<void> openInstallPermissionSettings() => _run(() async {
    if (state is! AppUpdatePermissionRequiredState) return;
    await _dependencies.installer.openPermissionSettings();
  });

  Future<void> refreshInstallStatus() => _run(() async {
    final current = state;
    if (current is! AppUpdateInstallingState) return;
    await _readInstallStatus(current.package, current.session);
  });

  Future<void> retryPendingInstall() => _run(() async {
    final current = state;
    if (current is! AppUpdateInstallingState ||
        current.status != UpdateInstallStatus.awaitingUserAction) {
      return;
    }
    await _replaceInstall(current.package, current.session);
  });

  Future<void> _run(Future<void> Function() operation) async {
    if (_operationActive || isClosed) return;
    _operationActive = true;
    try {
      await operation();
    } on AppFailure catch (failure) {
      emit(AppUpdateFailureState(failure.message));
    } on Object catch (error, stackTrace) {
      emit(AppUpdateFailureState(_unexpected(error, stackTrace)));
    } finally {
      _operationActive = false;
    }
  }

  void _emitState(AppUpdateState next) => emit(next);

  bool _foregroundCheckDue() {
    final lastCheckAt = _lastCheckAt;
    return lastCheckAt == null ||
        _clock().difference(lastCheckAt) >= foregroundCheckInterval;
  }
}

String _unexpected(Object error, StackTrace stackTrace) {
  return translatedBoundaryFailure(
    source: 'AppUpdateCubit',
    message: 'Could not complete the update operation.',
    error: error,
    stackTrace: stackTrace,
  ).message;
}
