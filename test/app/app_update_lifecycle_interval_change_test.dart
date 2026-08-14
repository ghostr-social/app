import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_update_lifecycle.dart';

void main() {
  testWidgets('an interval change replaces and disposal cancels the timer', (
    tester,
  ) async {
    var checks = 0;
    Widget subject(Duration interval) => MaterialApp(
      home: AppUpdateLifecycle(
        checkInterval: interval,
        onCheckDue: () async => checks += 1,
        onResumed: () async {},
        child: const Text('Video'),
      ),
    );

    await tester.pumpWidget(subject(const Duration(minutes: 2)));
    await tester.pump(const Duration(minutes: 1));
    expect(checks, 0);
    await tester.pumpWidget(subject(const Duration(minutes: 1)));
    await tester.pump(const Duration(minutes: 1));
    expect(checks, 1);
    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump(const Duration(minutes: 2));
    expect(checks, 1);
  });
}
