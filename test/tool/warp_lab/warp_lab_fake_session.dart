import 'package:flutter/material.dart';

import '../../../tool/warp_lab/warp_lab_destination.dart';
import '../../../tool/warp_lab/warp_lab_session.dart';

final class FakeWarpLabSession implements WarpLabSession {
  var closeCount = 0;

  @override
  Widget screen(WarpLabDestination destination) {
    return Scaffold(
      body: Semantics(
        label: destination.semanticLabel,
        child: Text('session:${destination.path}'),
      ),
    );
  }

  @override
  Future<void> close() async {
    closeCount += 1;
  }
}
