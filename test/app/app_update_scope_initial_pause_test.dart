import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_update_scope.dart';

import '../support/app_update_cubit_harness.dart';

void main() {
  testWidgets('a paused app defers its startup update check until resume', (
    tester,
  ) async {
    addTearDown(() {
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    });
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
    final harness = AppUpdateCubitHarness();
    final cubit = harness.build();

    await tester.pumpWidget(
      MaterialApp(
        home: AppUpdateScope(create: () => cubit, child: const Text('Video')),
      ),
    );
    await tester.pumpAndSettle();
    expect(harness.installedApp.calls, 0);
    expect(harness.catalog.calls, 0);

    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    await tester.pump();
    expect(harness.catalog.calls, 1);
    await tester.pumpWidget(const SizedBox.shrink());
  });
}
