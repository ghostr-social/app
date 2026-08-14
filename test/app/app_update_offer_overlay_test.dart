import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_offer_overlay.dart';

import '../support/app_update_cubit_harness.dart';

void main() {
  testWidgets('the update offer is nonmodal and keeps its child interactive', (
    tester,
  ) async {
    final harness = AppUpdateCubitHarness();
    final cubit = harness.build();
    final routes = _RouteCounter();
    var videoInteractions = 0;
    addTearDown(cubit.close);
    await cubit.start();

    await tester.pumpWidget(
      BlocProvider.value(
        value: cubit,
        child: MaterialApp(
          navigatorObservers: [routes],
          home: AppUpdateOfferOverlay(
            child: Scaffold(
              body: Align(
                alignment: Alignment.bottomLeft,
                child: TextButton(
                  onPressed: () => videoInteractions += 1,
                  child: const Text('Video remains interactive'),
                ),
              ),
            ),
          ),
        ),
      ),
    );

    expect(
      find.bySemanticsLabel('Ghostr 0.0.2 update available'),
      findsOneWidget,
    );
    expect(find.widgetWithText(FilledButton, 'Update'), findsOneWidget);
    expect(
      find.widgetWithText(TextButton, 'Skip this version'),
      findsOneWidget,
    );
    await tester.tap(find.text('Video remains interactive'));
    expect(videoInteractions, 1);
    expect(routes.pushes, 1);

    await tester.tap(find.text('Skip this version'));
    await tester.pump();
    expect(find.text('Skip this version'), findsNothing);
    expect(find.text('Video remains interactive'), findsOneWidget);
    expect(routes.pushes, 1);
  });
}

final class _RouteCounter extends NavigatorObserver {
  int pushes = 0;

  @override
  void didPush(Route<dynamic> route, Route<dynamic>? previousRoute) {
    pushes += 1;
  }
}
