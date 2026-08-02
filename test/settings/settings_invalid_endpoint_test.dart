import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/presentation/settings_cubit.dart';

import '../support/fake_app_settings_repository.dart';

void main() {
  test('rejects invalid relay and Blossom endpoint edits', () async {
    final cubit = SettingsCubit(
      FakeAppSettingsRepository(AppSettings.defaults()),
    );
    addTearDown(cubit.close);
    await cubit.load();

    cubit.addRelay('http://not-a-relay.example');
    expect(cubit.state.notice, 'Enter a valid ws:// or wss:// relay URL.');
    cubit.clearNotice();
    cubit.addBlossomServer('http://insecure.example');

    expect(
      cubit.state.notice,
      'Enter a valid HTTPS Blossom server URL.',
    );
  });
}
