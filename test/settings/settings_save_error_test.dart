import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

import '../support/fake_app_settings_repository.dart';
import '../support/settings_screen_harness.dart';

void main() {
  testWidgets('shows a safe save error and re-enables settings',
      (tester) async {
    final repository = _FailingSettingsRepository();
    await tester.pumpWidget(settingsScreenHarness(repository));
    await tester.pumpAndSettle();

    await tester.scrollUntilVisible(
      find.text('Save settings'),
      300,
      scrollable: find.byType(Scrollable).first,
    );
    await tester.tap(find.text('Save settings'));
    await tester.pumpAndSettle();

    expect(find.text('Could not save settings.'), findsOneWidget);
    final button = tester.widget<ElevatedButton>(
      find.widgetWithText(ElevatedButton, 'Save settings'),
    );
    expect(button.onPressed, isNotNull);
  });
}

class _FailingSettingsRepository extends FakeAppSettingsRepository {
  _FailingSettingsRepository() : super(AppSettings.defaults());

  @override
  Future<void> save(AppSettings value) {
    throw const AppFailure('Could not save settings.');
  }
}
