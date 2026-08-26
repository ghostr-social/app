import 'dart:async';

import 'package:flutter_test/flutter_test.dart';

import '../../../tool/warp_lab/warp_lab_bootstrap.dart';
import '../../../tool/warp_lab/warp_lab_destination.dart';
import '../../../tool/warp_lab/warp_lab_session.dart';

void main() {
  testWidgets('direct route shows its startup state while session loads', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    final pending = Completer<WarpLabSession>();
    await tester.pumpWidget(
      WarpLabBootstrap(
        initialRoute: WarpLabDestination.rapidSwipes.path,
        loadSession: (_) => pending.future,
      ),
    );

    expect(find.bySemanticsLabel('Starting Rapid swipes'), findsOneWidget);
    semantics.dispose();
  });
}
