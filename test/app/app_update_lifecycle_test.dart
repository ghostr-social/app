import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_update_lifecycle.dart';

void main() {
  testWidgets('forwards foreground resumes to the updater', (tester) async {
    var resumes = 0;
    await tester.pumpWidget(
      MaterialApp(
        home: AppUpdateLifecycle(
          onCheckDue: () async {},
          onResumed: () async => resumes += 1,
          child: const Text('Ghostr'),
        ),
      ),
    );

    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    await tester.pump();

    expect(resumes, 1);
  });
}
