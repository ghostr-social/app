import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_update_lifecycle.dart';

void main() {
  testWidgets('periodic checks run only while the app is active', (
    tester,
  ) async {
    var checks = 0;
    var resumes = 0;
    await tester.pumpWidget(
      MaterialApp(
        home: AppUpdateLifecycle(
          checkInterval: const Duration(minutes: 1),
          onCheckDue: () async => checks += 1,
          onResumed: () async => resumes += 1,
          child: const Text('Video'),
        ),
      ),
    );

    await tester.pump(const Duration(seconds: 59));
    expect(checks, 0);
    await tester.pump(const Duration(seconds: 1));
    expect(checks, 1);

    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
    await tester.pump(const Duration(minutes: 2));
    expect(checks, 1);
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    await tester.pump();
    expect(resumes, 1);
    await tester.pump(const Duration(minutes: 1));
    expect(checks, 2);

    await tester.pumpWidget(const SizedBox.shrink());
  });
}
