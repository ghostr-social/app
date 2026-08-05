import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';

import '../support/settings_form_harness.dart';

void main() {
  testWidgets('search relays can be reviewed, added, and removed',
      (tester) async {
    final harness = SettingsFormHarness(AppSettings.defaults());
    await harness.pump(tester);

    expect(find.text('Search relays'), findsOneWidget);
    expect(find.text('wss://nostr.wine'), findsOneWidget);

    await tester.tap(find.text('Add search relay'));
    expect(harness.searchRelayAdds, 1);

    await tester.tap(find.byTooltip('Remove wss://nostr.wine'));
    expect(harness.removedSearchRelays, [RelayUrl.parse('wss://nostr.wine')]);
  });

  testWidgets('search relay actions are disabled while saving', (tester) async {
    final harness = SettingsFormHarness(AppSettings.defaults(), isSaving: true);
    await harness.pump(tester);

    await tester.tap(find.text('Add search relay'), warnIfMissed: false);
    expect(harness.searchRelayAdds, 0);

    final remove = tester.widget<IconButton>(
      find.ancestor(
        of: find.byTooltip('Remove wss://nostr.wine'),
        matching: find.byType(IconButton),
      ),
    );
    expect(remove.onPressed, isNull);
  });
}
