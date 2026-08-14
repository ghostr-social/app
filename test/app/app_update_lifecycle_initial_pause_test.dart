import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_update_lifecycle.dart';

void main() {
  testWidgets('an initially paused app does not schedule update polling', (
    tester,
  ) async {
    var checks = 0;
    addTearDown(() {
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    });
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
    await tester.pumpWidget(
      MaterialApp(
        home: AppUpdateLifecycle(
          checkInterval: const Duration(minutes: 1),
          onCheckDue: () async => checks += 1,
          onResumed: () async {},
          child: const Text('Video'),
        ),
      ),
    );

    await tester.pump(const Duration(minutes: 2));
    expect(checks, 0);
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    await tester.pump();
    await tester.pump(const Duration(minutes: 1));
    expect(checks, 1);
  });
}
