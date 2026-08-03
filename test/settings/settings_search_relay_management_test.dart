import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/presentation/settings_cubit.dart';

import '../support/fake_app_settings_repository.dart';

void main() {
  test('adds, rejects, and removes NIP-50 search relays', () async {
    final defaults = AppSettings.defaults();
    final cubit = SettingsCubit(FakeAppSettingsRepository(defaults));
    addTearDown(cubit.close);
    await cubit.load();

    cubit.addSearchRelay('wss://search.example');
    expect(
      cubit.state.settings!.searchRelays.map((relay) => relay.value),
      contains('wss://search.example'),
    );

    cubit.addSearchRelay('not-a-relay');
    expect(cubit.state.notice, 'Enter a valid ws:// or wss:// relay URL.');
    expect(
      cubit.state.settings!.searchRelays,
      hasLength(defaults.searchRelays.length + 1),
    );

    cubit.removeSearchRelay(defaults.searchRelays.first);
    expect(
      cubit.state.settings!.searchRelays.map((relay) => relay.value),
      isNot(contains(defaults.searchRelays.first.value)),
    );
    expect(cubit.state.settings!.relays, defaults.relays);
  });
}
