import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

import '../support/fakes.dart';
import '../support/settings_screen_harness.dart';

void main() {
  testWidgets('flips the hide watched setting when the switch is tapped',
      (tester) async {
    final repository = FakeAppSettingsRepository(AppSettings.defaults());
    await tester.pumpWidget(settingsScreenHarness(repository));
    await tester.pumpAndSettle();
    await tester.scrollUntilVisible(
      find.byKey(const Key('hide-watched-field')),
      300,
      scrollable: find.byType(Scrollable).first,
    );
    await tester.ensureVisible(find.byKey(const Key('hide-watched-field')));
    await tester.pumpAndSettle();

    await tester.tap(find.byKey(const Key('hide-watched-field')));
    await tester.pumpAndSettle();

    final toggle = tester.widget<SwitchListTile>(
      find.byKey(const Key('hide-watched-field')),
    );
    expect(toggle.value, isFalse);

    await tester.scrollUntilVisible(
      find.text('Save settings'),
      300,
      scrollable: find.byType(Scrollable).first,
    );
    await tester.tap(find.text('Save settings'));
    await tester.pumpAndSettle();
    expect(repository.savedSettings?.hideWatchedVideos, isFalse);
  });
}
