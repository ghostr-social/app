import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';

import '../support/settings_screen_harness.dart';

void main() {
  testWidgets('disables every settings edit while a save is pending', (
    tester,
  ) async {
    final repository = _PendingSettingsRepository();
    await tester.pumpWidget(
      settingsScreenHarness(repository, onCheckForUpdates: () {}),
    );
    await tester.pumpAndSettle();
    await tester.scrollUntilVisible(
      find.text('Save settings'),
      300,
      scrollable: find.byType(Scrollable).first,
    );

    await tester.tap(find.text('Save settings'));
    await tester.pump();
    final removeFinder = find.byWidgetPredicate(
      (widget) =>
          widget is IconButton &&
          widget.tooltip == 'Remove wss://relay.damus.io',
    );
    await tester.scrollUntilVisible(
      removeFinder,
      -300,
      scrollable: find.byType(Scrollable).first,
    );
    await tester.pumpAndSettle();

    final add = tester.widget<OutlinedButton>(
      find.widgetWithText(OutlinedButton, 'Add relay'),
    );
    final remove = tester.widget<IconButton>(removeFinder);
    expect(add.onPressed, isNull);
    expect(remove.onPressed, isNull);
    await tester.scrollUntilVisible(
      find.byKey(const Key('inventory-budget-field')),
      300,
      scrollable: find.byType(Scrollable).first,
    );
    final budget = tester.widget<DropdownButtonFormField<VideoInventoryBudget>>(
      find.byKey(const Key('inventory-budget-field')),
    );
    expect(budget.onChanged, isNull);
    await tester.scrollUntilVisible(
      find.byKey(const Key('automatic-update-checks-field')),
      300,
      scrollable: find.byType(Scrollable).first,
    );
    final checks = tester.widget<SwitchListTile>(
      find.byKey(const Key('automatic-update-checks-field')),
    );
    final downloads = tester
        .widget<DropdownButtonFormField<UpdateDownloadPolicy>>(
          find.byKey(const Key('update-download-policy-field')),
        );
    final install = tester.widget<SwitchListTile>(
      find.byKey(const Key('automatic-update-install-field')),
    );
    final checkNow = tester.widget<OutlinedButton>(
      find.byKey(const Key('check-for-updates-button')),
    );
    expect(checks.onChanged, isNull);
    expect(downloads.onChanged, isNull);
    expect(install.onChanged, isNull);
    expect(checkNow.onPressed, isNull);
    repository.release.complete();
  });
}

class _PendingSettingsRepository implements AppSettingsRepository {
  final release = Completer<void>();

  @override
  Future<AppSettings> load() async => AppSettings.defaults();

  @override
  Future<void> save(AppSettings value) => release.future;
}
