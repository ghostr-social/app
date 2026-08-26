import 'package:flutter_test/flutter_test.dart';

import '../../../tool/warp_lab/warp_lab_bootstrap.dart';
import '../../../tool/warp_lab/warp_lab_destination.dart';
import 'warp_lab_fake_session.dart';

void main() {
  testWidgets('lab menu opens one selected WARP test session', (tester) async {
    final semantics = tester.ensureSemantics();
    final loaded = <WarpLabDestination>[];

    await tester.pumpWidget(
      WarpLabBootstrap(
        initialRoute: WarpLabDestination.menu.path,
        loadSession: (destination) async {
          loaded.add(destination);
          return FakeWarpLabSession();
        },
      ),
    );
    await tester.pump();

    expect(loaded, isEmpty);
    expect(
      find.text('Choose one route per app launch. Relaunch to switch routes.'),
      findsOneWidget,
    );
    for (final destination in WarpLabDestination.tests) {
      expect(find.text(destination.title), findsOneWidget);
    }

    await tester.tap(find.text(WarpLabDestination.rapidSwipes.title));
    await tester.pump();
    await tester.pump();

    expect(loaded, [WarpLabDestination.rapidSwipes]);
    expect(
      find.bySemanticsLabel(RegExp('^WARP rapid swipes test feed')),
      findsOneWidget,
    );
    semantics.dispose();
  });
}
