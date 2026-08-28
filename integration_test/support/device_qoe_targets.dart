const deviceStartupTarget = Duration(seconds: 2);
const deviceFocusSwitchTarget = Duration(milliseconds: 1500);
const deviceProtectedTransitionTarget = Duration(milliseconds: 500);
const deviceRapidSwipeGestureTarget = Duration(milliseconds: 16);
const deviceRapidSwipeDistanceFraction = 0.23;
const deviceRapidSwipeCadence = Duration(milliseconds: 150);
const deviceRapidSwipeMaximumInterval = Duration(milliseconds: 300);
const deviceHeldResponseRecoveryTarget = Duration(seconds: 2);
const deviceManifestRetryStartupTarget = Duration(seconds: 4);
const deviceRebufferTarget = 0.01;

bool deviceRapidCadenceIsWithinTarget(Iterable<Duration> intervals) {
  return intervals.isNotEmpty &&
      intervals.every(
        (interval) =>
            interval >= Duration.zero &&
            interval <= deviceRapidSwipeMaximumInterval,
      );
}

bool deviceReadyBurstRequiresPlaying(int index, int count) {
  return count > 0 && index == count - 1;
}
