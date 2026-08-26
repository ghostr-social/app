import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../../tool/warp_lab/warp_lab_bootstrap.dart';
import '../../../tool/warp_lab/warp_lab_destination.dart';
import '../../../tool/warp_lab/warp_lab_session.dart';

void main() {
  testWidgets('disposed lab absorbs a later startup failure', (tester) async {
    final pending = Completer<WarpLabSession>();
    await tester.pumpWidget(
      WarpLabBootstrap(
        initialRoute: WarpLabDestination.rapidSwipes.path,
        loadSession: (_) => pending.future,
      ),
    );

    await tester.pumpWidget(const SizedBox.shrink());
    pending.completeError(StateError('native startup failed'));
    await tester.pump();

    expect(tester.takeException(), isNull);
  });
}
