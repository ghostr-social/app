import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

import '../support/fakes.dart';
import '../support/settings_screen_harness.dart';

void main() {
  testWidgets('adds and saves a NIP-50 search relay', (tester) async {
    final repository = FakeAppSettingsRepository(AppSettings.defaults());
    await tester.pumpWidget(settingsScreenHarness(repository));
    await tester.pumpAndSettle();

    await tester.scrollUntilVisible(
      find.text('Add search relay'),
      300,
      scrollable: find.byType(Scrollable).first,
    );
    await tester.tap(find.text('Add search relay'));
    await tester.pumpAndSettle();
    await tester.enterText(
      find.byKey(const Key('search-relay-url-field')),
      'wss://search.band',
    );
    await tester.tap(find.widgetWithText(FilledButton, 'Add'));
    await tester.pumpAndSettle();
    await tester.scrollUntilVisible(
      find.text('Save settings'),
      300,
      scrollable: find.byType(Scrollable).first,
    );
    await tester.tap(find.text('Save settings'));
    await tester.pumpAndSettle();

    expect(
      repository.savedSettings?.searchRelays.map((relay) => relay.value),
      contains('wss://search.band'),
    );
    expect(
      repository.savedSettings?.relays.map((relay) => relay.value),
      isNot(contains('wss://search.band')),
    );
  });
}
