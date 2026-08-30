import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/device_qoe_targets.dart';

void main() {
  test('ready burst uses protected-transition and rapid-swipe budgets', () {
    expect(deviceProtectedTransitionTarget, const Duration(milliseconds: 500));
    expect(deviceRapidSwipeGestureTarget, const Duration(milliseconds: 16));
    expect(deviceRapidSwipeDistanceFraction, 0.23);
    expect(deviceRapidSwipeCadence, const Duration(milliseconds: 150));
    expect(deviceRapidSwipeMaximumInterval, const Duration(milliseconds: 300));
    expect(deviceCancellationWasteTargetBytes, 192 * 1024);
    expect(deviceRapidSwipeGestureTarget, lessThan(deviceRapidSwipeCadence));
    expect(
      deviceRapidCadenceIsWithinTarget([
        const Duration(milliseconds: 200),
        const Duration(milliseconds: 300),
      ]),
      isTrue,
    );
    expect(
      deviceRapidCadenceIsWithinTarget([const Duration(milliseconds: 301)]),
      isFalse,
    );
    expect(
      deviceRapidCadenceIsWithinTarget([const Duration(milliseconds: -1)]),
      isFalse,
    );
    expect(deviceReadyBurstRequiresPlaying(0, 3), isFalse);
    expect(deviceReadyBurstRequiresPlaying(1, 3), isFalse);
    expect(deviceReadyBurstRequiresPlaying(2, 3), isTrue);
  });
}
