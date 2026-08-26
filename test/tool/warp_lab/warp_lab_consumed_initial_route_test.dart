import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../../tool/warp_lab/warp_lab_bootstrap.dart';
import '../../../tool/warp_lab/warp_lab_destination.dart';
import 'warp_lab_fake_session.dart';

void main() {
  testWidgets('route host does not navigate its consumed platform route', (
    tester,
  ) async {
    final destination = WarpLabDestination.rapidSwipes;
    tester.binding.platformDispatcher.defaultRouteNameTestValue =
        destination.path;
    addTearDown(
      tester.binding.platformDispatcher.clearDefaultRouteNameTestValue,
    );

    await tester.pumpWidget(
      WarpLabBootstrap(
        initialRoute: destination.path,
        loadSession: (_) async => FakeWarpLabSession(),
      ),
    );
    await tester.pumpAndSettle();

    expect(tester.takeException(), isNull);
    final visibleText = tester
        .widgetList<Text>(find.byType(Text))
        .map((widget) => widget.data)
        .whereType<String>()
        .toList();
    expect(
      find.text('session:${destination.path}'),
      findsOneWidget,
      reason: 'Visible text: $visibleText',
    );
  });
}
