import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';

final class FailingRollbackSettingsRepository implements AppSettingsRepository {
  FailingRollbackSettingsRepository(this.settings);

  AppSettings settings;
  var _saveCount = 0;

  @override
  Future<AppSettings> load() async => settings;

  @override
  Future<void> save(AppSettings value) async {
    _saveCount += 1;
    if (_saveCount > 1) throw StateError('storage rollback failed');
    settings = value;
  }
}
