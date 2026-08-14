import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_update_scope.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';

import '../support/app_update_cubit_harness.dart';
import '../support/fake_update_offer_history_repository.dart';

void main() {
  testWidgets('a near-due resume does not postpone the six-hour check', (
    tester,
  ) async {
    var now = DateTime.utc(2026, 8, 14, 12);
    final history = FakeUpdateOfferHistoryRepository()
      ..lastDeclined = AndroidVersionCode(2);
    final harness = AppUpdateCubitHarness(offerHistory: history);
    final cubit = harness.build(clock: () => now);
    await tester.pumpWidget(
      MaterialApp(
        home: AppUpdateScope(create: () => cubit, child: const Text('Video')),
      ),
    );
    await tester.pump();
    expect(harness.catalog.calls, 1);

    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
    now = now.add(const Duration(hours: 5, minutes: 59));
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    await tester.pump();
    expect(harness.catalog.calls, 1);

    now = now.add(const Duration(minutes: 1));
    await tester.pump(const Duration(minutes: 1));
    expect(harness.catalog.calls, 2);
    await tester.pumpWidget(const SizedBox.shrink());
  });
}
