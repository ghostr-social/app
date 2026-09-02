part of 'warp_player_verified_rescue_scenario.dart';

extension _PlayerVerifiedRescueEvidence on _PlayerVerifiedRescueScenario {
  void verifyCandidateEvidence() {
    final state = journey.cubit.state as FeedLoaded;
    final intended = _deliveryId(state, 1);
    final structural = _deliveryId(state, 2);
    final ready = _deliveryId(state, 3);
    final preparation = journey.preparation.latest;
    final structuralAsset = _assetFor(preparation, structural);
    final readyAsset = _assetFor(preparation, ready);
    expect(bodyGate.isReached, isTrue);
    expect(bodyGate.timedOut, isFalse);
    _expectStalledIntended(intended);
    expect(structuralAsset.readiness.isStructurallyStartable, isTrue);
    expect(structuralAsset.readiness.isPlayerVerified, isFalse);
    expect(playerGate.blockedBuilds, greaterThan(0));
    expect(journey.playerStages.attemptsFor(structural), isEmpty);
    expect(readyAsset.readiness, PlaybackPreparationReadiness.ready);
    expect(readyAsset.authority, readyAuthority);
    expect(readyStage.isTerminal, isFalse);
    expect(readyStage.firstFrameAt, isNotNull);
  }

  Future<PlaybackFocus> swipeToRescue(WidgetTester tester) async {
    final cursor = journey.focusCursor;
    await journey.swipeUp(tester);
    final focus = await journey.waitForPublishedFocus(
      tester,
      3,
      afterSequence: cursor,
      cause: FeedFocusCause.transportRescue,
    );
    bodyGate.release();
    expect(bodyGate.timedOut, isFalse);
    final state = journey.cubit.state as FeedLoaded;
    expect(state.activeIndex, 3);
    expect(focus.rescue?.reason, FeedTransportRescueReason.etaTooLong);
    expect(focus.rescue?.rankDisplacement, 2);
    expect(focus.rescue?.wait, Duration.zero);
    await journey.waitForCaption(tester, 3);
    await waitForReadyCurrent(tester);
    return focus;
  }

  Future<void> verifyVisibleFrame(
    WidgetTester tester,
    PlaybackFocus focus,
  ) async {
    await journey.waitForFirstFrame(tester, focus);
    await journey.waitForPlaying(tester, focus);
    final presentation = journey.telemetry.probe.presentationFor(focus);
    expect(presentation, isNotNull);
    final stage = journey.playerStages.forPresentation(
      presentation!.session.deliveryId,
      presentation.elapsed,
    );
    expect(stage, same(readyStage));
    expect(stage?.firstFrameAt, isNotNull);
    expect(find.text('Video unavailable'), findsNothing);
    final position = journey.telemetry.probe.latestPositionFor(focus)!;
    await journey.pumpFor(tester, const Duration(seconds: 1));
    expect(
      journey.telemetry.probe.latestPositionFor(focus),
      greaterThan(position),
    );
  }

  void _expectStalledIntended(PlaybackDeliveryId deliveryId) {
    final matches = journey.graph.deliveryProbe.observations
        .map((item) => item.snapshot)
        .where((snapshot) => snapshot.deliveryId == deliveryId)
        .toList();
    expect(matches, isNotEmpty);
    expect(matches.last.phase, VideoDeliveryPhase.preparing);
    final eta = matches.last.eta;
    expect(eta, isNotNull);
    expect(eta!, greaterThan(const FeedReadySelector().grace));
  }
}

PlaybackDeliveryId _deliveryId(FeedLoaded state, int index) {
  return state.posts[index].media.playbackDeliveryId!;
}

WarpFeedCurrentPreparation _assetFor(
  WarpFeedPreparationObservation snapshot,
  PlaybackDeliveryId deliveryId,
) {
  return snapshot.upcoming.singleWhere(
    (asset) => asset.authority.deliveryId == deliveryId,
  );
}
