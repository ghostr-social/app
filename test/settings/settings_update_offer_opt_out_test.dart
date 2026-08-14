import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

import '../support/fake_app_settings_repository.dart';
import '../support/settings_screen_harness.dart';

void main() {
  testWidgets('users can opt out of automatic update offers', (tester) async {
    final repository = FakeAppSettingsRepository(AppSettings.defaults());
    await tester.pumpWidget(settingsScreenHarness(repository));
    await tester.pumpAndSettle();

    final field = find.byKey(const Key('automatic-update-checks-field'));
    await tester.scrollUntilVisible(
      field,
      300,
      scrollable: find.byType(Scrollable).first,
    );
    await Scrollable.ensureVisible(tester.element(field), alignment: 0.5);
    await tester.pumpAndSettle();
    expect(
      find.bySemanticsLabel(RegExp('Offer new app versions automatically')),
      findsOneWidget,
    );
    await tester.tap(field);
    await tester.scrollUntilVisible(
      find.byKey(const Key('save-settings-button')),
      300,
      scrollable: find.byType(Scrollable).first,
    );
    await tester.tap(find.byKey(const Key('save-settings-button')));
    await tester.pumpAndSettle();

    expect(repository.savedSettings!.updatePreferences.automaticChecks, false);
  });
}
