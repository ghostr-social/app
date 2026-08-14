import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/features/settings/presentation/settings_cubit.dart';
import 'package:ghostr/features/settings/presentation/settings_screen.dart';

import '../app_update/support/update_domain_fixture.dart';
import '../support/app_update_cubit_harness.dart';

void main() {
  testWidgets('a stale Settings action cannot accept an unseen release', (
    tester,
  ) async {
    var now = DateTime.utc(2026, 8, 14, 12);
    final harness = AppUpdateCubitHarness();
    final updates = harness.build(clock: () => now);
    addTearDown(updates.close);
    await updates.start();
    await tester.pumpWidget(
      MaterialApp(
        home: BlocProvider(
          create: (_) => SettingsCubit(harness.settings)..load(),
          child: SettingsScreen(appUpdateCubit: updates),
        ),
      ),
    );
    await tester.pumpAndSettle();
    final button = find.widgetWithText(FilledButton, 'Download update');
    await tester.scrollUntilVisible(
      button,
      300,
      scrollable: find.byType(Scrollable).first,
    );
    final staleAction = tester.widget<FilledButton>(button).onPressed!;

    harness.catalog.release = sampleStableRelease(
      versionName: '0.0.3',
      versionCode: 3,
    );
    now = now.add(AppUpdateCubit.foregroundCheckInterval);
    await updates.onPeriodicCheck();
    staleAction();
    await tester.pump(const Duration(milliseconds: 10));

    expect(harness.downloader.calls, 0);
    expect(
      (updates.state as AppUpdateOfferedState).release.versionCode.value,
      3,
    );
  });
}
