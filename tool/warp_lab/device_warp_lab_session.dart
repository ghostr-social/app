import 'dart:async';

import 'package:flutter/widgets.dart';

import '../../integration_test/support/warp_feed_device_runtime.dart';
import 'warp_lab_destination.dart';
import 'warp_lab_feed_surface.dart';
import 'warp_lab_profile.dart';
import 'warp_lab_session.dart';

final class DeviceWarpLabSession implements WarpLabSession {
  DeviceWarpLabSession._(this._runtime);

  static Future<DeviceWarpLabSession> start(
    WarpLabDestination destination,
  ) async {
    final profile = WarpLabProfile.forDestination(destination);
    final runtime = await WarpFeedDeviceRuntime.start(
      eventCount: profile.eventCount,
      validator: profile.validator,
      dataUsage: profile.dataUsage,
      responseChunkDelay: profile.responseChunkDelay,
    );
    return DeviceWarpLabSession._(runtime);
  }

  final WarpFeedDeviceRuntime _runtime;
  var _loadStarted = false;
  var _closed = false;

  @override
  Widget screen(WarpLabDestination destination) {
    if (!_loadStarted) {
      _loadStarted = true;
      unawaited(_runtime.graph.cubit.load());
    }
    return WarpLabFeedSurface(destination: destination, graph: _runtime.graph);
  }

  @override
  Future<void> close() async {
    if (_closed) return;
    _closed = true;
    await _runtime.close();
  }
}
