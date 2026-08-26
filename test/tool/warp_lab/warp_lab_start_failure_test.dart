import 'package:flutter_test/flutter_test.dart';

import '../../../tool/warp_lab/warp_lab_bootstrap.dart';
import '../../../tool/warp_lab/warp_lab_destination.dart';

void main() {
  testWidgets('session startup failure gives visible relaunch guidance', (
    tester,
  ) async {
    await tester.pumpWidget(
      WarpLabBootstrap(
        initialRoute: WarpLabDestination.rapidSwipes.path,
        loadSession: (_) async => throw StateError('native engine busy'),
      ),
    );
    await tester.pump();

    expect(find.text('WARP Lab could not start'), findsOneWidget);
    expect(
      find.text('Stop the lab process and relaunch this test route.'),
      findsOneWidget,
    );
  });
}
