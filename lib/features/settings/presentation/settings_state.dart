import 'package:ghostr/features/settings/domain/app_settings.dart';

enum SettingsStatus { loading, ready, failure }

sealed class SettingsState {
  const SettingsState();

  const factory SettingsState.loading() = SettingsLoading;

  const factory SettingsState.failure(String message) = SettingsFailure;

  factory SettingsState.ready(
    AppSettings settings, {
    bool isSaving = false,
    String? notice,
  }) {
    return SettingsReady(settings, isSaving: isSaving, notice: notice);
  }

  SettingsStatus get status;
  AppSettings? get settings => null;
  bool get isSaving => false;
  String? get message => null;
  String? get notice => null;
}

final class SettingsLoading extends SettingsState {
  const SettingsLoading();

  @override
  SettingsStatus get status => SettingsStatus.loading;
}

final class SettingsFailure extends SettingsState {
  const SettingsFailure(this.failureMessage);

  final String failureMessage;

  @override
  SettingsStatus get status => SettingsStatus.failure;

  @override
  String get message => failureMessage;
}

final class SettingsReady extends SettingsState {
  const SettingsReady(
    this.readySettings, {
    this.isSaving = false,
    this.notice,
  });

  final AppSettings readySettings;
  @override
  final bool isSaving;
  @override
  final String? notice;

  @override
  SettingsStatus get status => SettingsStatus.ready;

  @override
  AppSettings get settings => readySettings;

  SettingsReady saving() => SettingsReady(readySettings, isSaving: true);

  SettingsReady withNotice(String value) {
    return SettingsReady(readySettings, notice: value);
  }

  SettingsReady withoutNotice() => SettingsReady(readySettings);
}
