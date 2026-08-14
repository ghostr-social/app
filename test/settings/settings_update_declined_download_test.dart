import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';
import 'package:ghostr/features/settings/presentation/settings_cubit.dart';
import 'package:ghostr/features/settings/presentation/settings_screen.dart';

import '../support/app_update_cubit_harness.dart';

void main() {
  testWidgets('Settings can download a version after its offer is skipped', (
    tester,
  ) async {
    final harness = AppUpdateCubitHarness(
      preferences: const AppUpdatePreferences(
        automaticChecks: true,
        downloadPolicy: UpdateDownloadPolicy.anyNetwork,
        automaticInstall: false,
      ),
    );
    final updates = harness.build();
    addTearDown(updates.close);
    await updates.start();
    final offered = updates.state as AppUpdateOfferedState;
    await updates.declineOffer(offered.release.versionCode);
    await tester.pumpWidget(
      MaterialApp(
        home: BlocProvider(
          create: (_) => SettingsCubit(harness.settings)..load(),
          child: SettingsScreen(appUpdateCubit: updates),
        ),
      ),
    );
    await tester.pumpAndSettle();
    final download = find.widgetWithText(FilledButton, 'Download update');
    await tester.scrollUntilVisible(
      download,
      300,
      scrollable: find.byType(Scrollable).first,
    );
    await Scrollable.ensureVisible(tester.element(download), alignment: 0.5);
    await tester.pumpAndSettle();

    await tester.tap(download);
    await tester.pumpAndSettle();

    expect(harness.downloader.calls, 1);
  });
}
