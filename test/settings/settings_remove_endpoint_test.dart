import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/presentation/settings_cubit.dart';

import '../support/fake_app_settings_repository.dart';

void main() {
  test('removes configured relay and Blossom endpoints', () async {
    final defaults = AppSettings.defaults();
    final cubit = SettingsCubit(FakeAppSettingsRepository(defaults));
    addTearDown(cubit.close);
    await cubit.load();

    cubit.removeRelay(defaults.relays.first);
    cubit.removeBlossomServer(defaults.blossomServers.first);

    expect(cubit.state.settings!.relays, hasLength(defaults.relays.length - 1));
    expect(cubit.state.settings!.blossomServers, isEmpty);
  });
}
