import 'package:ghostr/core/storage/preference_storage_guard.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';
import 'package:ghostr/features/app_update/domain/update_offer_history_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

final class LocalUpdateOfferHistoryRepository
    implements UpdateOfferHistoryRepository {
  LocalUpdateOfferHistoryRepository(this._preferences);

  static const _lastDeclinedKey = 'ghostr.updates.lastDeclinedVersionCode';

  final SharedPreferences _preferences;

  @override
  Future<AndroidVersionCode?> readLastDeclinedVersion() {
    return guardPreferenceStorage('Could not read update history.', _read);
  }

  AndroidVersionCode? _read() {
    final value = _preferences.get(_lastDeclinedKey);
    if (value is! int || value < 1 || value > AndroidVersionCode.maximum) {
      return null;
    }
    return AndroidVersionCode(value);
  }

  @override
  Future<void> recordDeclinedVersion(AndroidVersionCode version) async {
    final current = await readLastDeclinedVersion();
    if (current != null && current.compareTo(version) >= 0) return;
    await requirePreferenceWrite(
      'Could not save update history.',
      () => _preferences.setInt(_lastDeclinedKey, version.value),
    );
  }
}
