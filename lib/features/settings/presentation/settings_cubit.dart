import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';
import 'package:ghostr/features/settings/domain/blossom_server_url.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/settings/presentation/settings_state.dart';

export 'settings_state.dart';

class SettingsCubit extends DisposalSafeCubit<SettingsState> {
  SettingsCubit(this._repository) : super(const SettingsState.loading());

  final AppSettingsRepository _repository;

  Future<void> load() async {
    emit(const SettingsState.loading());
    try {
      emit(SettingsState.ready(await _repository.load()));
    } on AppFailure catch (failure) {
      emit(SettingsState.failure(failure.message));
    } on Object catch (error, stackTrace) {
      final failure = translatedBoundaryFailure(
        source: 'SettingsCubit.load',
        message: 'Could not load settings.',
        error: error,
        stackTrace: stackTrace,
      );
      emit(SettingsState.failure(failure.message));
    }
  }

  void addRelay(String raw) {
    final settings = _editableSettings;
    if (settings == null) return;
    final relay = RelayUrl.tryParse(raw);
    if (relay == null) {
      _notice('Enter a valid ws:// or wss:// relay URL.');
      return;
    }
    final relays = <RelayUrl>{...settings.relays, relay}.toList();
    emit(SettingsState.ready(settings.copyWith(relays: relays)));
  }

  void addBlossomServer(String raw) {
    final settings = _editableSettings;
    if (settings == null) return;
    final server = BlossomServerUrl.tryParse(raw);
    if (server == null) {
      _notice('Enter a valid HTTPS Blossom server URL.');
      return;
    }
    final servers = <BlossomServerUrl>{
      ...settings.blossomServers,
      server,
    }.toList();
    emit(SettingsState.ready(settings.copyWith(blossomServers: servers)));
  }

  void removeRelay(RelayUrl relay) {
    final settings = _editableSettings;
    if (settings == null) return;
    final relays = settings.relays.where((item) => item != relay).toList();
    emit(SettingsState.ready(settings.copyWith(relays: relays)));
  }

  void addSearchRelay(String raw) {
    final settings = _editableSettings;
    if (settings == null) return;
    final relay = RelayUrl.tryParse(raw);
    if (relay == null) {
      _notice('Enter a valid ws:// or wss:// relay URL.');
      return;
    }
    final relays = <RelayUrl>{...settings.searchRelays, relay}.toList();
    emit(SettingsState.ready(settings.copyWith(searchRelays: relays)));
  }

  void removeSearchRelay(RelayUrl relay) {
    final settings = _editableSettings;
    if (settings == null) return;
    final relays =
        settings.searchRelays.where((item) => item != relay).toList();
    emit(SettingsState.ready(settings.copyWith(searchRelays: relays)));
  }

  void changeDataUsage(DataUsageLevel dataUsage) {
    final settings = _editableSettings;
    if (settings == null) return;
    emit(SettingsState.ready(settings.copyWith(dataUsage: dataUsage)));
  }

  void removeBlossomServer(BlossomServerUrl server) {
    final settings = _editableSettings;
    if (settings == null) return;
    final servers =
        settings.blossomServers.where((item) => item != server).toList();
    emit(SettingsState.ready(settings.copyWith(blossomServers: servers)));
  }

  void changeBudget(VideoInventoryBudget budget) {
    final settings = _editableSettings;
    if (settings == null) return;
    emit(SettingsState.ready(settings.copyWith(inventoryBudget: budget)));
  }

  void changeHideWatchedVideos(bool hideWatchedVideos) {
    final settings = _editableSettings;
    if (settings == null) return;
    emit(SettingsState.ready(
      settings.copyWith(hideWatchedVideos: hideWatchedVideos),
    ));
  }

  Future<void> save() async {
    final current = state;
    if (current is! SettingsReady || current.isSaving) return;
    final settings = current.settings;
    emit(current.saving());
    try {
      await _repository.save(settings);
      _notice('Settings saved. Blossom server changes apply after restart.');
    } on AppFailure catch (failure) {
      _notice(failure.message);
    } on Object catch (error, stackTrace) {
      final failure = translatedBoundaryFailure(
        source: 'SettingsCubit.save',
        message: 'Could not save settings.',
        error: error,
        stackTrace: stackTrace,
      );
      _notice(failure.message);
    }
  }

  void clearNotice() {
    final current = state;
    if (current is SettingsReady && current.notice != null) {
      emit(current.withoutNotice());
    }
  }

  void _notice(String message) {
    final current = state;
    if (current is SettingsReady) emit(current.withNotice(message));
  }

  AppSettings? get _editableSettings {
    final current = state;
    return current is SettingsReady && !current.isSaving
        ? current.settings
        : null;
  }
}
