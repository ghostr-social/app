import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

import '../support/fakes.dart';
import '../support/settings_screen_harness.dart';

void main() {
  testWidgets('cancels a settings endpoint dialog without editing settings',
      (tester) async {
    final repository = FakeAppSettingsRepository(AppSettings.defaults());
    await tester.pumpWidget(settingsScreenHarness(repository));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Add relay'));
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(TextButton, 'Cancel'));
    await tester.pumpAndSettle();

    expect(find.byKey(const Key('relay-url-field')), findsNothing);
    expect(repository.savedSettings, isNull);
  });
}
