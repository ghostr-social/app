import 'package:flutter_test/flutter_test.dart';

import '../../../tool/warp_lab/warp_lab_bootstrap.dart';
import '../../../tool/warp_lab/warp_lab_destination.dart';
import 'warp_lab_fake_session.dart';

void main() {
  testWidgets('direct rapid-swipe route starts its selected lab session', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    final loaded = <WarpLabDestination>[];
    final session = FakeWarpLabSession();

    await tester.pumpWidget(
      WarpLabBootstrap(
        initialRoute: WarpLabDestination.rapidSwipes.path,
        loadSession: (destination) async {
          loaded.add(destination);
          return session;
        },
      ),
    );
    await tester.pumpAndSettle();

    expect(loaded, [WarpLabDestination.rapidSwipes]);
    expect(
      find.bySemanticsLabel(RegExp('^WARP rapid swipes test feed')),
      findsOneWidget,
    );
    expect(find.text('session:/warp/rapid-swipes'), findsOneWidget);
    semantics.dispose();
  });
}
