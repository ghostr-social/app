part of 'warp_feed_playback_journey.dart';

typedef WarpReadyWindow = ({
  WarpPlanEvidence plan,
  WarpFeedPreparationObservation snapshot,
});

typedef _WarpWindowQuery = ({
  BigInt generation,
  int minimumDepth,
  int afterRevision,
  int afterSequence,
  PlaybackDeliveryId? currentDeliveryId,
});

extension WarpFeedPlaybackJourneySwipe on WarpFeedPlaybackJourney {
  List<String> futureRemotePaths(int count) {
    final state = cubit.state;
    if (state is! FeedLoaded) throw StateError('Feed is not loaded.');
    final posts = state.posts.skip(state.activeIndex + 1).take(count).toList();
    if (posts.length != count) throw RangeError.value(count);
    return posts
        .map((post) => Uri.parse(post.media.remoteUrl!).path)
        .toList(growable: false);
  }

  List<String> remotePathsFor(Iterable<WarpFeedCurrentPreparation> assets) {
    final state = cubit.state;
    if (state is! FeedLoaded) throw StateError('Feed is not loaded.');
    return assets
        .map((asset) {
          final post = state.posts.singleWhere(
            (post) =>
                post.media.playbackDeliveryId == asset.authority.deliveryId,
          );
          return Uri.parse(post.media.remoteUrl!).path;
        })
        .toList(growable: false);
  }

  Future<WarpReadyWindow> waitForReadyWindow(
    WidgetTester tester,
    BigInt generation, {
    required PlaybackDeliveryId currentDeliveryId,
    int minimumDepth = 1,
    int afterSequence = 0,
  }) async {
    return _waitForWindow(tester, (
      generation: generation,
      minimumDepth: minimumDepth,
      afterRevision: 0,
      afterSequence: afterSequence,
      currentDeliveryId: currentDeliveryId,
    ));
  }

  Future<List<PlaybackFocus>> swipeForward(
    WidgetTester tester, {
    required int count,
    required int afterSequence,
    Duration cadence = const Duration(milliseconds: 500),
  }) async {
    final expected = events
        .skip(1)
        .take(count)
        .map((event) => event.id)
        .toList();
    final watch = Stopwatch()..start();
    for (var index = 0; index < count; index += 1) {
      await swipeUp(tester);
      final due = cadence * (index + 1);
      final remaining = due - watch.elapsed;
      if (remaining > Duration.zero) await pumpFor(tester, remaining);
    }
    await _wait(tester, () => _hasUserFocusOrder(expected, afterSequence));
    return _userFocusesAfter(afterSequence).take(count).toList();
  }

  bool isReadyIn(
    WarpFeedPreparationObservation snapshot,
    PlaybackFocus focused,
  ) {
    final session = telemetry.probe.activationFor(focused);
    if (session == null) return false;
    final stage = playerStages.preparedFor(
      session.deliveryId,
      focused.startedAt,
    );
    return stage != null &&
        snapshot.has(stage.authority, PlaybackPreparationReadiness.ready);
  }

  Future<WarpReadyWindow> waitForReplenishment(
    WidgetTester tester,
    PlaybackFocus focused, {
    required int afterRevision,
  }) async {
    final generation = focus.generationFor(focused)!;
    final session = telemetry.probe.activationFor(focused)!;
    return _waitForWindow(tester, (
      generation: generation,
      minimumDepth: 1,
      afterRevision: afterRevision,
      afterSequence: 0,
      currentDeliveryId: session.deliveryId,
    ));
  }

  Future<WarpReadyWindow> _waitForWindow(
    WidgetTester tester,
    _WarpWindowQuery query,
  ) async {
    final watch = Stopwatch()..start();
    while (watch.elapsed < const Duration(seconds: 15)) {
      final match = await _matchingWindow(query);
      if (match != null) return match;
      await _tickAndSample(tester);
    }
    await reportSchedulingEvidence();
    fail(_timeoutEvidence(const Duration(seconds: 15)));
  }

  Future<WarpReadyWindow?> _matchingWindow(_WarpWindowQuery query) async {
    for (final snapshot in preparation.observations.reversed) {
      final revision = snapshot.revision.toInt();
      if (revision <= query.afterRevision) continue;
      if (snapshot.sequence <= query.afterSequence) continue;
      if (snapshot.contiguousReadyDepth < query.minimumDepth) continue;
      if (query.currentDeliveryId != null &&
          snapshot.currentDeliveryId != query.currentDeliveryId) {
        continue;
      }
      final plan = await _planAt(revision, query.generation);
      if (plan == null ||
          plan.plan.readyReserve.target > snapshot.contiguousReadyDepth) {
        continue;
      }
      return (plan: plan, snapshot: snapshot);
    }
    return null;
  }

  Future<WarpPlanEvidence?> _planAt(int revision, BigInt generation) async {
    final page = await evidence.page(afterRevision: revision - 1, limit: 1);
    if (page.planPage.records.isEmpty) return null;
    final plan = page.planPage.records.single;
    if (plan.revision != revision || !plan.coversFocusGeneration(generation)) {
      return null;
    }
    return plan;
  }
}
