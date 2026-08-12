import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/build_production_dependencies.dart';

void main() {
  test('enables direct binary updates only for Android apps', () {
    expect(
      supportsDirectAppUpdates(isWeb: false, platform: TargetPlatform.android),
      isTrue,
    );
    expect(
      supportsDirectAppUpdates(isWeb: false, platform: TargetPlatform.iOS),
      isFalse,
    );
    expect(
      supportsDirectAppUpdates(isWeb: true, platform: TargetPlatform.android),
      isFalse,
    );
  });
}
