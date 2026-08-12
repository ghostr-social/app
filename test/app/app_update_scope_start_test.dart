import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_update_scope.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../support/app_update_cubit_harness.dart';

void main() {
  testWidgets('starts update checks without delaying its child', (
    tester,
  ) async {
    final harness = AppUpdateCubitHarness(
      preferences: const AppUpdatePreferences(
        automaticChecks: true,
        downloadPolicy: UpdateDownloadPolicy.manual,
        automaticInstall: false,
      ),
    );
    final cubit = harness.build();

    await tester.pumpWidget(
      MaterialApp(
        home: AppUpdateScope(
          create: () => cubit,
          child: const Text('Session gate'),
        ),
      ),
    );

    expect(find.text('Session gate'), findsOneWidget);
    await tester.pump();
    expect(harness.catalog.calls, 1);
    expect(cubit.state, isA<AppUpdateAvailableState>());
    await tester.pumpWidget(const SizedBox.shrink());
  });
}
