import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';
import 'package:ghostr/features/settings/presentation/settings_cubit.dart';
import 'package:ghostr/features/settings/presentation/settings_screen.dart';

import '../support/app_update_cubit_harness.dart';

void main() {
  testWidgets('Settings queues Update during a quiet offer refresh', (
    tester,
  ) async {
    var now = DateTime.utc(2026, 8, 14, 12);
    final harness = AppUpdateCubitHarness(
      preferences: const AppUpdatePreferences(
        automaticChecks: true,
        downloadPolicy: UpdateDownloadPolicy.anyNetwork,
        automaticInstall: false,
      ),
    );
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
    await Scrollable.ensureVisible(tester.element(button), alignment: 0.5);
    await tester.pumpAndSettle();
    final gate = Completer<void>();
    harness.catalog.beforeResult = gate.future;

    now = now.add(AppUpdateCubit.foregroundCheckInterval);
    final refresh = updates.onPeriodicCheck();
    await tester.pump();
    await tester.tap(button);
    await tester.pump();

    expect(
      (updates.state as AppUpdateOfferedState).pendingAction,
      AppUpdateOfferAction.accepting,
    );
    expect(tester.widget<FilledButton>(button).onPressed, isNull);
    expect(find.bySemanticsLabel(RegExp('Starting update')), findsOneWidget);
    gate.complete();
    for (var attempt = 0; attempt < 10; attempt += 1) {
      await tester.pump(const Duration(milliseconds: 10));
      if (harness.downloader.calls == 1) break;
    }
    await refresh;
    expect(harness.downloader.calls, 1);
  });
}
