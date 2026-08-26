import 'package:flutter/widgets.dart';

import 'warp_lab_destination.dart';

abstract interface class WarpLabSession {
  Widget screen(WarpLabDestination destination);

  Future<void> close();
}

typedef WarpLabSessionLoader =
    Future<WarpLabSession> Function(WarpLabDestination destination);
