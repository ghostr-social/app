part of 'warp_feed_playback_journey.dart';

extension WarpFeedPlaybackJourneyReport on WarpFeedPlaybackJourney {
  Future<void> reportSchedulingEvidence() async {
    _reportPreparationEvidence();
    final overview = await evidence.page(afterRevision: 0, limit: 1);
    final latest = overview.planPage.latestRetainedRevision;
    final page = await evidence.page(
      afterRevision: (latest - 12).clamp(0, latest),
      limit: 12,
    );
    for (final plan in page.planPage.records) {
      reportPlan(plan);
    }
    final decisions = await evidence.decisions();
    for (final decision
        in decisions.records.reversed.take(20).toList().reversed) {
      debugPrint(_decisionEvidence(decision));
    }
  }

  void _reportPreparationEvidence() {
    for (final snapshot
        in preparation.observations.reversed.take(12).toList().reversed) {
      final upcoming = snapshot.upcoming
          .map(
            (asset) =>
                '${asset.authority.deliveryId.value}:${asset.readiness.name}',
          )
          .join('|');
      debugPrint(
        'WARP_PREPARATION revision=${snapshot.revision} '
        'sequence=${snapshot.sequence} current=${snapshot.currentDeliveryId?.value} '
        'contiguous=${snapshot.contiguousReadyDepth} upcoming=$upcoming',
      );
    }
  }

  void reportPlan(WarpPlanEvidence plan) {
    final reserve = plan.plan.readyReserve;
    final allocations = plan.plan.allocations
        .map(
          (item) => '${item.postId}:${item.start}-${item.end}:${item.reason}',
        )
        .join('|');
    debugPrint(
      'WARP_PLAN revision=${plan.revision} focus=${plan.focusGeneration} '
      'covers=${plan.focusCoversFrom} '
      'current=${plan.currentPostId} target=${reserve.target} '
      'candidates=${reserve.candidateCount} ready=${reserve.ready} '
      'structural=${reserve.structural} coverage_ms=${reserve.readyCoverageMs} '
      'next=${plan.plan.nextReserveStatus} allocations=$allocations',
    );
  }

  void reportParallelPreparation(
    WarpReadyWindow window,
    ProgressiveRangedRequestPair pair,
  ) {
    final first = pair.first;
    final second = pair.second;
    debugPrint(
      'WARP_PARALLEL revision=${window.plan.revision} '
      'paths=${first.path},${second.path} '
      'byte_intervals_ms=${first.firstByteAt!.inMilliseconds}-'
      '${first.lastByteAt!.inMilliseconds},'
      '${second.firstByteAt!.inMilliseconds}-${second.lastByteAt!.inMilliseconds} '
      'bytes=${first.servedBytes},${second.servedBytes}',
    );
  }

  void reportStartup(PlaybackFocus focus) {
    final firstFrame = telemetry.probe.firstFrameLatency(focus)?.inMilliseconds;
    final progress = telemetry.probe.playingLatency(focus)?.inMilliseconds;
    debugPrint(
      'WARP_QOE startup_ms=$firstFrame progress_ms=$progress '
      '${_stageEvidence(focus)} origin=${_originEvidence()}',
    );
  }

  void reportFinal(PlaybackFocus focus) {
    final firstFrame = telemetry.probe.firstFrameLatency(focus)?.inMilliseconds;
    final progress = telemetry.probe.playingLatency(focus)?.inMilliseconds;
    debugPrint(
      'WARP_QOE focus_switch_ms=$firstFrame progress_ms=$progress '
      '${_stageEvidence(focus)}',
    );
  }

  void reportBurst(
    WarpReadyWindow initial,
    WarpReadyWindow replenished,
    List<PlaybackFocus> focuses,
    PlaybackFocus next,
  ) {
    final intervals = <int>[];
    for (var index = 1; index < focuses.length; index += 1) {
      intervals.add(
        (focuses[index].startedAt - focuses[index - 1].startedAt)
            .inMilliseconds,
      );
    }
    final refillMs =
        (replenished.snapshot.elapsed - focuses.last.startedAt).inMilliseconds;
    debugPrint(
      'WARP_BURST target=${initial.plan.plan.readyReserve.target} '
      'ready=${initial.snapshot.contiguousReadyDepth} '
      'focus_intervals_ms=${intervals.join(',')} replenish_ms=$refillMs',
    );
    for (final focus in focuses) {
      reportFinal(focus);
    }
    reportFinal(next);
  }

  String _stageEvidence(PlaybackFocus focus) {
    final presentation = telemetry.probe.presentationFor(focus);
    if (presentation == null) return 'stages=unavailable';
    final deliveryId = presentation.session.deliveryId;
    final player = playerStages.latestFor(
      deliveryId,
      noLaterThan: presentation.elapsed,
    );
    final authority = player?.authority;
    final structural = authority == null
        ? null
        : preparation.firstStructurallyStartableAt(authority);
    final ready = authority == null
        ? null
        : preparation.firstAt(authority, PlaybackPreparationReadiness.ready);
    return 'rust_structural_startable_ms=${_deltaMs(structural, focus)} '
        'rust_ready_ms=${_deltaMs(ready, focus)} '
        'structural_depth=${preparation.maximumStructuralDepth} '
        'ready_depth=${preparation.maximumReadyDepth} '
        'player_prepare_ms=${_deltaMs(player?.preparedAt, focus)} '
        'initialize_start_ms=${_deltaMs(player?.initializingAt, focus)} '
        'initialized_ms=${_deltaMs(player?.initializedAt, focus)} '
        'native_frame_ms=${_deltaMs(player?.firstFrameAt, focus)} '
        'presented_ms=${_deltaMs(presentation.elapsed, focus)}';
  }

  String _originEvidence() {
    return resources.origin.requests
        .map((request) {
          final range = request.range;
          final span = range == null ? 'full' : '${range.start}-${range.end}';
          final finished = request.finishedAt?.inMilliseconds ?? 'open';
          return '${request.method}:${request.path}:$span:'
              'bytes=${request.servedBytes}:state=${request.outcome.name}:'
              'time_ms=${request.startedAt.inMilliseconds}-$finished';
        })
        .join(',');
  }
}

String _decisionEvidence(WarpDecisionRecord record) {
  final selected = record.selected;
  final executed = record.executed;
  return 'WARP_DECISION sequence=${record.sequence} '
      'action=${record.chosenActionId} outcome=${record.outcome.status} '
      'detail=${record.outcome.failureClass ?? record.outcome.claimRefusal} '
      'bytes=${record.outcome.bytes} elapsed_ms=${record.outcome.elapsedMs} '
      'selected=${selected?.kind}:${selected?.command}:${selected?.postId}:'
      '${selected?.sourceId}:${selected?.start}-${selected?.end} '
      'executed=${executed?.postId}:${executed?.sourceId}:'
      '${executed?.start}-${executed?.end}';
}

String _deltaMs(Duration? observed, PlaybackFocus focus) {
  return observed == null
      ? 'na'
      : '${(observed - focus.startedAt).inMilliseconds}';
}
