import 'package:ghostr/features/settings/domain/app_settings_repository.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

class FakeAppSettingsRepository implements AppSettingsRepository {
  FakeAppSettingsRepository(this.settings);

  AppSettings settings;
  AppSettings? savedSettings;

  @override
  Future<AppSettings> load() async => settings;

  @override
  Future<void> save(AppSettings value) async {
    settings = value;
    savedSettings = value;
  }
}
