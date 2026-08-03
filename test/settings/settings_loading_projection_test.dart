import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/presentation/settings_state.dart';

void main() {
  test('loading settings expose no editable or failure data', () {
    final state = SettingsState.loading();

    expect(state.status, SettingsStatus.loading);
    expect(state.settings, isNull);
    expect(state.isSaving, isFalse);
    expect(state.message, isNull);
    expect(state.notice, isNull);
  });
}
