import 'package:ghostr/features/settings/domain/app_settings.dart';

enum SettingsStatus { loading, ready, failure }

class SettingsState {
  const SettingsState._({
    required this.status,
    this.settings,
    this.isSaving = false,
    this.message,
    this.notice,
  });

  const SettingsState.loading() : this._(status: SettingsStatus.loading);

  const SettingsState.failure(String message)
      : this._(status: SettingsStatus.failure, message: message);

  const SettingsState.ready(
    AppSettings settings, {
    bool isSaving = false,
    String? notice,
  }) : this._(
          status: SettingsStatus.ready,
          settings: settings,
          isSaving: isSaving,
          notice: notice,
        );

  final SettingsStatus status;
  final AppSettings? settings;
  final bool isSaving;
  final String? message;
  final String? notice;

  SettingsState saving() => SettingsState.ready(settings!, isSaving: true);

  SettingsState edited(AppSettings value) => SettingsState.ready(value);

  SettingsState withNotice(String value) {
    return SettingsState.ready(settings!, notice: value);
  }

  SettingsState withoutNotice() => SettingsState.ready(settings!);
}
