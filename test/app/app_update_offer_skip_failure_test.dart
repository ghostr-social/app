import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_offer_overlay.dart';

import '../support/app_update_cubit_harness.dart';
import '../support/fake_update_offer_history_repository.dart';

void main() {
  testWidgets('a durable skip shows pending and retryable error states', (
    tester,
  ) async {
    final gate = Completer<void>();
    final history = FakeUpdateOfferHistoryRepository(
      writeFailure: StateError('disk unavailable'),
    )..beforeWrite = gate.future;
    final harness = AppUpdateCubitHarness(offerHistory: history);
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

    await tester.tap(find.text('Skip this version'));
    await tester.pump();
    expect(find.bySemanticsLabel('Saving skipped version'), findsOneWidget);
    expect(
      tester.widget<TextButton>(find.byType(TextButton)).onPressed,
      isNull,
    );
    expect(
      tester.widget<FilledButton>(find.byType(FilledButton)).onPressed,
      isNull,
    );
    gate.complete();
    await tester.pumpAndSettle();

    expect(
      find.text('Could not skip this version. Please try again.'),
      findsOne,
    );
    expect(
      find.bySemanticsLabel('Could not skip this version. Please try again.'),
      findsOneWidget,
    );
    expect(
      tester.widget<TextButton>(find.byType(TextButton)).onPressed,
      isNotNull,
    );
  });
}
