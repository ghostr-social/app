import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('MainActivity owns update and incoming-video bridges', () {
    final activity = File(
      'android/app/src/main/kotlin/social/ghostr/MainActivity.kt',
    ).readAsStringSync();

    expect(activity, contains('AndroidAppUpdateBridge('));
    expect(activity, contains('IncomingVideoShareBridge('));
    expect(activity, contains('IncomingVideoShareActivityLifecycle('));
    expect(activity, contains('shareLifecycle.configureEngine()'));
    expect(activity, contains('shareLifecycle.receive(intent)'));
    expect(activity, contains('shareLifecycle.savedCaptureId'));
    expect(activity, contains('shareLifecycle.acknowledge(generation'));
    expect(activity, contains('appUpdateBridge?.dispose()'));
    expect(activity, contains('incomingVideoShareBridge?.dispose()'));
  });
}
