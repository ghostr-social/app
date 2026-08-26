import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../../tool/warp_lab/warp_lab_bootstrap.dart';
import '../../../tool/warp_lab/warp_lab_destination.dart';
import '../../../tool/warp_lab/warp_lab_session.dart';
import 'warp_lab_fake_session.dart';

void main() {
  testWidgets('session finishing after disposal is closed exactly once', (
    tester,
  ) async {
    final pending = Completer<WarpLabSession>();
    final session = FakeWarpLabSession();
    await tester.pumpWidget(
      WarpLabBootstrap(
        initialRoute: WarpLabDestination.rapidSwipes.path,
        loadSession: (_) => pending.future,
      ),
    );

    await tester.pumpWidget(const SizedBox.shrink());
    pending.complete(session);
    await tester.pump();

    expect(session.closeCount, 1);
  });
}
