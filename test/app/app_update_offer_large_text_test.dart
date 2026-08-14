import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_offer_overlay.dart';

import '../support/app_update_cubit_harness.dart';

void main() {
  testWidgets('the update offer remains usable with large text', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(320, 568);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final harness = AppUpdateCubitHarness();
    final cubit = harness.build();
    addTearDown(cubit.close);
    await cubit.start();

    await tester.pumpWidget(
      MaterialApp(
        builder: (context, child) => MediaQuery(
          data: MediaQuery.of(
            context,
          ).copyWith(textScaler: const TextScaler.linear(2.5)),
          child: child!,
        ),
        home: AppUpdateOfferOverlay(
          cubit: cubit,
          child: const Scaffold(body: Text('Video')),
        ),
      ),
    );

    expect(tester.takeException(), isNull);
    expect(find.bySemanticsLabel('Update'), findsOneWidget);
    expect(find.bySemanticsLabel('Skip this version'), findsOneWidget);
  });
}
