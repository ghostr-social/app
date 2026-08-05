import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';

final class PartiallyFailingSettingsRepository
    implements AppSettingsRepository {
  PartiallyFailingSettingsRepository(this.settings, this.failure);

  AppSettings settings;
  final Object failure;
  var _saveCount = 0;

  @override
  Future<AppSettings> load() async => settings;

  @override
  Future<void> save(AppSettings value) async {
    settings = value;
    _saveCount += 1;
    if (_saveCount == 1) throw failure;
  }
}
