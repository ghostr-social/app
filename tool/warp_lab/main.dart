import 'package:flutter/material.dart';

import 'device_warp_lab_session.dart';
import 'warp_lab_bootstrap.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  final route = WidgetsBinding.instance.platformDispatcher.defaultRouteName;
  runApp(
    WarpLabBootstrap(
      initialRoute: route,
      loadSession: DeviceWarpLabSession.start,
    ),
  );
}
