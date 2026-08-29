part of 'warp_player_verified_rescue_scenario.dart';

extension _PlayerVerifiedRescueWait on _PlayerVerifiedRescueScenario {
  Future<void> waitForCandidateEvidence(WidgetTester tester) {
    return _waitForEvidence(tester, _captureCandidateEvidence, 'candidate');
  }

  Future<void> waitForReadyCurrent(WidgetTester tester) {
    return _waitForEvidence(tester, _isReadyCurrent, 'current-ready');
  }

  Future<void> _waitForEvidence(
    WidgetTester tester,
    bool Function() condition,
    String label,
  ) async {
    final watch = Stopwatch()..start();
    while (watch.elapsed < const Duration(seconds: 15)) {
      if (condition()) return;
      await journey.pumpFor(tester, const Duration(milliseconds: 70));
    }
    await journey.reportSchedulingEvidence();
    fail('WARP $label evidence timed out; gateTimedOut=${bodyGate.timedOut}');
  }

  bool _captureCandidateEvidence() {
    final context = _candidateContext();
    if (context == null) return false;
    final intended = _deliveryId(context.state, 1);
    final structural = _deliveryId(context.state, 2);
    final ready = _deliveryId(context.state, 3);
    final structuralAsset = _maybeAssetFor(context.snapshot, structural);
    final readyAsset = _maybeAssetFor(context.snapshot, ready);
    if (!_matchesCandidates(
      intended,
      structural,
      structuralAsset,
      readyAsset,
    )) {
      return false;
    }
    final stage = _readyStageFor(journey, ready, readyAsset!.authority);
    if (stage == null) return false;
    readyStage = stage;
    return true;
  }

  ({FeedLoaded state, WarpFeedPreparationObservation snapshot})?
  _candidateContext() {
    if (!bodyGate.isReached || bodyGate.isReleased || bodyGate.timedOut) {
      return null;
    }
    if (journey.preparation.observations.isEmpty) return null;
    final state = journey.cubit.state;
    if (state is! FeedLoaded || state.posts.length < 4) return null;
    return (state: state, snapshot: journey.preparation.latest);
  }

  bool _matchesCandidates(
    PlaybackDeliveryId intended,
    PlaybackDeliveryId structural,
    WarpFeedCurrentPreparation? structuralAsset,
    WarpFeedCurrentPreparation? readyAsset,
  ) {
    if (!_hasStalledIntended(journey, intended)) return false;
    if (!_isUntouchedStructural(structural, structuralAsset)) return false;
    return readyAsset?.readiness == PlaybackPreparationReadiness.ready;
  }

  bool _isUntouchedStructural(
    PlaybackDeliveryId deliveryId,
    WarpFeedCurrentPreparation? asset,
  ) {
    if (asset == null || !asset.readiness.isStructurallyStartable) return false;
    if (asset.readiness.isPlayerVerified || playerGate.blockedBuilds == 0) {
      return false;
    }
    return journey.playerStages.attemptsFor(deliveryId).isEmpty;
  }

  bool _isReadyCurrent() {
    if (journey.preparation.observations.isEmpty) return false;
    final snapshot = journey.preparation.latest;
    final current = snapshot.current;
    return snapshot.currentDeliveryId == readyAuthority.deliveryId &&
        current?.authority == readyAuthority &&
        current?.readiness == PlaybackPreparationReadiness.ready;
  }
}
