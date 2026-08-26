import 'dart:async';

import 'package:flutter_test/flutter_test.dart';

import '../../../tool/warp_lab/warp_lab_bootstrap.dart';
import '../../../tool/warp_lab/warp_lab_destination.dart';
import '../../../tool/warp_lab/warp_lab_session.dart';
import 'warp_lab_fake_session.dart';

void main() {
  testWidgets('queued route taps start exactly one native session', (
    tester,
  ) async {
    final pending = Completer<WarpLabSession>();
    var startCount = 0;

    await tester.pumpWidget(
      WarpLabBootstrap(
        initialRoute: WarpLabDestination.menu.path,
        loadSession: (_) {
          startCount += 1;
          return pending.future;
        },
      ),
    );

    final route = find.text(WarpLabDestination.rapidSwipes.title);
    await tester.tap(route);
    await tester.tap(route);

    expect(startCount, 1);
    pending.complete(FakeWarpLabSession());
    await tester.pumpAndSettle();
  });
}
