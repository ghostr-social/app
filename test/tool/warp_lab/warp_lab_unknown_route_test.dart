import 'package:flutter_test/flutter_test.dart';

import '../../../tool/warp_lab/warp_lab_bootstrap.dart';
import '../../../tool/warp_lab/warp_lab_destination.dart';
import 'warp_lab_fake_session.dart';

void main() {
  testWidgets('unknown lab route explains the error without starting Rust', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    var loadCount = 0;

    await tester.pumpWidget(
      WarpLabBootstrap(
        initialRoute: '/warp/not-a-test',
        loadSession: (_) async {
          loadCount += 1;
          return FakeWarpLabSession();
        },
      ),
    );
    await tester.pump();

    expect(
      find.bySemanticsLabel(RegExp('^Unknown WARP Lab route')),
      findsOneWidget,
    );
    expect(find.text('/warp/not-a-test'), findsOneWidget);
    expect(loadCount, 0);

    await tester.tap(find.text('Open WARP Lab'));
    await tester.pump();

    expect(find.text(WarpLabDestination.feedPlayback.title), findsOneWidget);
    expect(loadCount, 0);
    semantics.dispose();
  });
}
