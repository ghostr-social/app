import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

import '../support/fake_app_settings_repository.dart';
import '../support/settings_screen_harness.dart';

void main() {
  testWidgets('configures, checks, and saves automatic app updates', (
    tester,
  ) async {
    final repository = FakeAppSettingsRepository(AppSettings.defaults());
    var checks = 0;
    await tester.pumpWidget(
      settingsScreenHarness(repository, onCheckForUpdates: () => checks += 1),
    );
    await tester.pumpAndSettle();

    await _show(tester, const Key('automatic-update-checks-field'));
    expect(find.text('App updates'), findsOneWidget);
    expect(
      find.bySemanticsLabel(RegExp('Check for updates automatically')),
      findsOneWidget,
    );
    await tester.tap(find.byKey(const Key('automatic-update-checks-field')));
    await _chooseAnyNetwork(tester);
    await _show(tester, const Key('automatic-update-install-field'));
    await tester.tap(find.byKey(const Key('automatic-update-install-field')));
    await tester.tap(find.byKey(const Key('check-for-updates-button')));
    expect(checks, 1);

    await _show(tester, const Key('save-settings-button'));
    await tester.tap(find.byKey(const Key('save-settings-button')));
    await tester.pumpAndSettle();

    final saved = repository.savedSettings!.updatePreferences;
    expect(saved.automaticChecks, isFalse);
    expect(saved.downloadPolicy, UpdateDownloadPolicy.anyNetwork);
    expect(saved.automaticInstall, isFalse);
  });
}

Future<void> _chooseAnyNetwork(WidgetTester tester) async {
  await _show(tester, const Key('update-download-policy-field'));
  await tester.tap(find.byKey(const Key('update-download-policy-field')));
  await tester.pumpAndSettle();
  await tester.tap(find.text('Wi-Fi or mobile data').last);
  await tester.pumpAndSettle();
}

Future<void> _show(WidgetTester tester, Key key) async {
  await tester.scrollUntilVisible(
    find.byKey(key),
    250,
    scrollable: find.byType(Scrollable).first,
  );
  await tester.pumpAndSettle();
}
