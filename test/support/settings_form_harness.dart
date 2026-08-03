import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/settings/presentation/settings_form.dart';
import 'package:ghostr/features/settings/presentation/settings_form_actions.dart';

/// Pumps a [SettingsForm] with recording callbacks for widget tests.
class SettingsFormHarness {
  SettingsFormHarness(this.settings, {this.isSaving = false});

  final AppSettings settings;
  final bool isSaving;
  final List<RelayUrl> removedSearchRelays = <RelayUrl>[];
  final List<DataUsageLevel> dataUsageChanges = <DataUsageLevel>[];
  int searchRelayAdds = 0;

  /// Uses a tall viewport so every settings section is built and hittable.
  Future<void> pump(WidgetTester tester) async {
    tester.view.physicalSize = const Size(800, 2600);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    await tester.pumpWidget(build());
  }

  Widget build() {
    return MaterialApp(
      home: Scaffold(
        body: SettingsForm(
          settings: settings,
          isSaving: isSaving,
          actions: SettingsFormActions(
            relays: RelaySettingsActions(onAdd: () {}, onRemove: (_) {}),
            searchRelays: RelaySettingsActions(
              onAdd: () => searchRelayAdds += 1,
              onRemove: removedSearchRelays.add,
            ),
            blossom: BlossomSettingsActions(onAdd: () {}, onRemove: (_) {}),
            onBudgetChanged: (_) {},
            onDataUsageChanged: dataUsageChanges.add,
            onHideWatchedChanged: (_) {},
            onSave: () {},
          ),
        ),
      ),
    );
  }
}
