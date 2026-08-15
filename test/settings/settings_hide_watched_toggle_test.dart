import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

import '../support/fakes.dart';
import '../support/settings_screen_harness.dart';

void main() {
  testWidgets('watched exclusion is an immutable policy, not a toggle', (
    tester,
  ) async {
    final repository = FakeAppSettingsRepository(AppSettings.defaults());
    await tester.pumpWidget(settingsScreenHarness(repository));
    await tester.pumpAndSettle();
    await tester.scrollUntilVisible(
      find.byKey(const Key('watched-video-policy')),
      300,
      scrollable: find.byType(Scrollable).first,
    );
    expect(find.byKey(const Key('hide-watched-field')), findsNothing);
    expect(
      find.text(
        'Watched videos stay out of For You and search until you '
        'clear your watch history.',
      ),
      findsOneWidget,
    );
  });
}
