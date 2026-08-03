import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

import '../support/fakes.dart';
import '../support/settings_screen_harness.dart';

void main() {
  testWidgets('saves relay connections and the video inventory budget',
      (tester) async {
    final repository = FakeAppSettingsRepository(AppSettings.defaults());

    await tester.pumpWidget(settingsScreenHarness(repository));
    await tester.pumpAndSettle();
    await _addRelay(tester);
    await _selectBudget(tester);
    await _save(tester);

    final saved = repository.savedSettings;
    expect(saved?.relays.map((relay) => relay.value),
        contains('wss://relay.primal.net'));
    expect(saved?.inventoryBudget, VideoInventoryBudget.fourGigabytes);
    expect(
      find.text(
        'Settings saved. Restart Ghostr to apply connection and cache changes.',
      ),
      findsOneWidget,
    );
  });
}

Future<void> _addRelay(WidgetTester tester) async {
  await tester.tap(find.text('Add relay'));
  await tester.pumpAndSettle();
  await tester.enterText(
    find.byKey(const Key('relay-url-field')),
    'wss://relay.primal.net',
  );
  await tester.tap(find.widgetWithText(FilledButton, 'Add'));
  await tester.pumpAndSettle();
}

Future<void> _selectBudget(WidgetTester tester) async {
  final field = find.byKey(const Key('inventory-budget-field'));
  await tester.scrollUntilVisible(
    field,
    200,
    scrollable: find.byType(Scrollable).first,
  );
  await tester.pumpAndSettle();
  await tester.tap(field);
  await tester.pumpAndSettle();
  await tester.tap(find.text('4 GB').last);
  await tester.pumpAndSettle();
}

Future<void> _save(WidgetTester tester) async {
  await tester.scrollUntilVisible(
    find.text('Save settings'),
    300,
    scrollable: find.byType(Scrollable).first,
  );
  await tester.tap(find.text('Save settings'));
  await tester.pumpAndSettle();
}
