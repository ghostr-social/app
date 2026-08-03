import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

import '../support/settings_form_harness.dart';

void main() {
  testWidgets('the data usage level is selectable from settings',
      (tester) async {
    final harness = SettingsFormHarness(AppSettings.defaults());
    await harness.pump(tester);

    expect(find.text('Balanced'), findsOneWidget);

    await tester.tap(find.byKey(const Key('data-usage-field')));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Aggressive').last);
    await tester.pumpAndSettle();

    expect(harness.dataUsageChanges, [DataUsageLevel.aggressive]);
  });
}
