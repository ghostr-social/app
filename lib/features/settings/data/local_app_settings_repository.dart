import 'package:ghostr/core/storage/preference_storage_guard.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/blossom_server_url.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:shared_preferences/shared_preferences.dart';

class LocalAppSettingsRepository implements AppSettingsRepository {
  LocalAppSettingsRepository(this._preferences);

  static const _relaysKey = 'ghostr.settings.relays';
  static const _inventoryBudgetKey = 'ghostr.settings.inventoryBudget';
  static const _blossomServersKey = 'ghostr.settings.blossomServers';
  static const _searchRelaysKey = 'ghostr.settings.searchRelays';
  static const _dataUsageKey = 'ghostr.settings.dataUsage';
  static const _automaticUpdateChecksKey =
      'ghostr.settings.updates.automaticChecks';
  static const _updateDownloadPolicyKey =
      'ghostr.settings.updates.downloadPolicy';
  static const _automaticUpdateInstallKey =
      'ghostr.settings.updates.automaticInstall';

  final SharedPreferences _preferences;

  @override
  Future<AppSettings> load() {
    return guardPreferenceStorage('Could not read app settings.', _load);
  }

  AppSettings _load() {
    final defaults = AppSettings.defaults();
    return AppSettings(
      relays: _loadRelayList(_relaysKey, defaults.relays),
      inventoryBudget: _loadBudget(defaults.inventoryBudget),
      blossomServers: _loadBlossomServers(defaults.blossomServers),
      searchRelays: _loadRelayList(_searchRelaysKey, defaults.searchRelays),
      dataUsage: _loadDataUsage(defaults.dataUsage),
      updatePreferences: _loadUpdatePreferences(defaults.updatePreferences),
    );
  }

  @override
  Future<void> save(AppSettings settings) async {
    await _saveConnections(settings);
    await _saveMedia(settings);
    await _saveBehavior(settings);
    await _saveUpdatePreferences(settings.updatePreferences);
  }

  Future<void> _saveConnections(AppSettings settings) async {
    await requirePreferenceWrite(
      'Could not save app settings.',
      () => _preferences.setStringList(
        _relaysKey,
        settings.relays.map((relay) => relay.value).toList(),
      ),
    );
    await requirePreferenceWrite(
      'Could not save app settings.',
      () => _preferences.setStringList(
        _searchRelaysKey,
        settings.searchRelays.map((relay) => relay.value).toList(),
      ),
    );
  }

  Future<void> _saveMedia(AppSettings settings) async {
    await requirePreferenceWrite(
      'Could not save app settings.',
      () => _preferences.setString(
        _inventoryBudgetKey,
        settings.inventoryBudget.name,
      ),
    );
    await requirePreferenceWrite(
      'Could not save app settings.',
      () => _preferences.setStringList(
        _blossomServersKey,
        settings.blossomServers.map((server) => server.value).toList(),
      ),
    );
  }

  Future<void> _saveBehavior(AppSettings settings) async {
    await requirePreferenceWrite(
      'Could not save app settings.',
      () => _preferences.setString(_dataUsageKey, settings.dataUsage.name),
    );
  }

  Future<void> _saveUpdatePreferences(AppUpdatePreferences updates) async {
    await requirePreferenceWrite(
      'Could not save app settings.',
      () => _preferences.setBool(
        _automaticUpdateChecksKey,
        updates.automaticChecks,
      ),
    );
    await requirePreferenceWrite(
      'Could not save app settings.',
      () => _preferences.setString(
        _updateDownloadPolicyKey,
        updates.downloadPolicy.name,
      ),
    );
    await requirePreferenceWrite(
      'Could not save app settings.',
      () => _preferences.setBool(
        _automaticUpdateInstallKey,
        updates.automaticInstall,
      ),
    );
  }

  List<RelayUrl> _loadRelayList(String key, List<RelayUrl> defaults) {
    final saved = _preferences.getStringList(key);
    if (saved == null) return defaults;
    return saved.map(RelayUrl.tryParse).whereType<RelayUrl>().toSet().toList();
  }

  DataUsageLevel _loadDataUsage(DataUsageLevel fallback) {
    final name = _preferences.getString(_dataUsageKey);
    return DataUsageLevel.values
            .where((level) => level.name == name)
            .firstOrNull ??
        fallback;
  }

  AppUpdatePreferences _loadUpdatePreferences(AppUpdatePreferences fallback) {
    final savedPolicy = _preferences.getString(_updateDownloadPolicyKey);
    final policy =
        UpdateDownloadPolicy.values
            .where((value) => value.name == savedPolicy)
            .firstOrNull ??
        fallback.downloadPolicy;
    return AppUpdatePreferences(
      automaticChecks:
          _preferences.getBool(_automaticUpdateChecksKey) ??
          fallback.automaticChecks,
      downloadPolicy: policy,
      automaticInstall:
          _preferences.getBool(_automaticUpdateInstallKey) ??
          fallback.automaticInstall,
    );
  }

  List<BlossomServerUrl> _loadBlossomServers(List<BlossomServerUrl> defaults) {
    final saved = _preferences.getStringList(_blossomServersKey);
    if (saved == null) return defaults;
    return saved
        .map(BlossomServerUrl.tryParse)
        .whereType<BlossomServerUrl>()
        .toSet()
        .toList();
  }

  VideoInventoryBudget _loadBudget(VideoInventoryBudget fallback) {
    final name = _preferences.getString(_inventoryBudgetKey);
    return VideoInventoryBudget.values
            .where((budget) => budget.name == name)
            .firstOrNull ??
        fallback;
  }
}
