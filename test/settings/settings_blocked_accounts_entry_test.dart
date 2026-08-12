import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

import '../support/fakes.dart';
import '../support/settings_screen_harness.dart';

void main() {
  testWidgets('opens blocked accounts from the settings entry tile',
      (tester) async {
    var opened = 0;
    final repository = FakeAppSettingsRepository(AppSettings.defaults());
    await tester.pumpWidget(
      settingsScreenHarness(
        repository,
        onOpenBlockedAccounts: () => opened++,
      ),
    );
    await tester.pumpAndSettle();
    await tester.scrollUntilVisible(
      find.byKey(const Key('blocked-accounts-entry')),
      300,
      scrollable: find.byType(Scrollable).first,
    );
    await tester.ensureVisible(find.byKey(const Key('blocked-accounts-entry')));
    await tester.pumpAndSettle();

    await tester.tap(find.byKey(const Key('blocked-accounts-entry')));
    await tester.pumpAndSettle();

    expect(opened, 1);
  });
}
