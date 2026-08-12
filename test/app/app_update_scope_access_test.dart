import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_update_scope.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../support/app_update_cubit_harness.dart';

void main() {
  testWidgets('keeps one accessible updater across parent rebuilds', (
    tester,
  ) async {
    final harness = AppUpdateCubitHarness(
      preferences: const AppUpdatePreferences(
        automaticChecks: false,
        downloadPolicy: UpdateDownloadPolicy.manual,
        automaticInstall: false,
      ),
    );
    final cubit = harness.build();
    AppUpdateCubit? exposed;

    Widget subject(String label) => MaterialApp(
      home: AppUpdateScope(
        create: () => cubit,
        child: Builder(
          builder: (context) {
            exposed = AppUpdateScope.maybeOf(context);
            return Text(label);
          },
        ),
      ),
    );

    await tester.pumpWidget(subject('Initial'));
    expect(exposed, same(cubit));

    await tester.pumpWidget(subject('Rebuilt'));
    expect(find.text('Rebuilt'), findsOneWidget);
    expect(exposed, same(cubit));

    await tester.pumpWidget(const SizedBox.shrink());
  });
}
