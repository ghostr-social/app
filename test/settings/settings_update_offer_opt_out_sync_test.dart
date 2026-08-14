import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/features/settings/presentation/settings_cubit.dart';
import 'package:ghostr/features/settings/presentation/settings_screen.dart';

import '../support/app_update_cubit_harness.dart';

void main() {
  testWidgets('saving the offer opt-out hides the current automatic offer', (
    tester,
  ) async {
    final harness = AppUpdateCubitHarness();
    final updates = harness.build();
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
    final field = find.byKey(const Key('automatic-update-checks-field'));
    final scrollable = find.byType(Scrollable).first;
    await tester.scrollUntilVisible(field, 300, scrollable: scrollable);
    await Scrollable.ensureVisible(tester.element(field), alignment: 0.5);
    await tester.pumpAndSettle();
    await tester.tap(field);
    await tester.scrollUntilVisible(
      find.byKey(const Key('save-settings-button')),
      300,
      scrollable: scrollable,
    );
    await tester.tap(find.byKey(const Key('save-settings-button')));
    await tester.pumpAndSettle();

    expect(updates.state, isA<AppUpdateAvailableState>());
  });
}
