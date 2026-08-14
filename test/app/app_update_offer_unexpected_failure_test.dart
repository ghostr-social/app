import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_offer_overlay.dart';

import '../support/app_update_cubit_harness.dart';

void main() {
  testWidgets('an unexpected Update failure stays safe and retryable', (
    tester,
  ) async {
    final harness = AppUpdateCubitHarness();
    harness.network.failure = StateError('transport internals');
    final cubit = harness.build();
    addTearDown(cubit.close);
    await cubit.start();
    await tester.pumpWidget(
      MaterialApp(
        home: AppUpdateOfferOverlay(
          cubit: cubit,
          child: const Scaffold(body: Text('Video')),
        ),
      ),
    );

    await tester.tap(find.widgetWithText(FilledButton, 'Update'));
    await tester.pumpAndSettle();

    expect(
      find.text('Could not complete the update operation.'),
      findsOneWidget,
    );
    expect(find.widgetWithText(FilledButton, 'Update'), findsOneWidget);
  });
}
