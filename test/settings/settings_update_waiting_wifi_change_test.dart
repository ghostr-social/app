import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/network_connection_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';
import 'package:ghostr/features/settings/presentation/settings_cubit.dart';
import 'package:ghostr/features/settings/presentation/settings_screen.dart';

import '../support/app_update_cubit_harness.dart';

void main() {
  testWidgets('saving mobile downloads resumes an accepted update', (
    tester,
  ) async {
    final harness = AppUpdateCubitHarness(
      connection: NetworkConnection.other,
      preferences: const AppUpdatePreferences(
        automaticChecks: true,
        downloadPolicy: UpdateDownloadPolicy.wifiOnly,
        automaticInstall: false,
      ),
    );
    final updates = harness.build();
    addTearDown(updates.close);
    await updates.start();
    await acceptCurrentUpdateOffer(updates);
    expect(updates.state, isA<AppUpdateWaitingForWifiState>());
    await tester.pumpWidget(
      MaterialApp(
        home: BlocProvider(
          create: (_) => SettingsCubit(harness.settings)..load(),
          child: SettingsScreen(appUpdateCubit: updates),
        ),
      ),
    );
    await tester.pumpAndSettle();
    final wifi = find.byKey(const Key('wifi-only-update-downloads-field'));
    final scrollable = find.byType(Scrollable).first;
    await tester.scrollUntilVisible(wifi, 300, scrollable: scrollable);
    await Scrollable.ensureVisible(tester.element(wifi), alignment: 0.5);
    await tester.pumpAndSettle();
    await tester.tap(wifi);
    final save = find.byKey(const Key('save-settings-button'));
    await tester.scrollUntilVisible(save, 300, scrollable: scrollable);
    await Scrollable.ensureVisible(tester.element(save), alignment: 0.5);
    await tester.pumpAndSettle();

    await tester.tap(save);
    await tester.pumpAndSettle();

    expect(harness.downloader.calls, 1);
    expect(updates.state, isA<AppUpdateReadyState>());
  });
}
