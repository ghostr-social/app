part of 'warp_long_session_scenario.dart';

extension _WarpLongSessionDecodedEvidence on _WarpLongSessionDriver {
  Future<Never> _failDecodedPlayback(
    PlaybackFocus focus,
    Duration timeout,
  ) async {
    final overview = await graph.evidence.page(limit: 1);
    final latest = overview.planPage.latestRetainedRevision;
    final page = await graph.evidence.page(
      afterRevision: (latest - 12).clamp(0, latest),
      limit: 12,
    );
    final decisions = await graph.evidence.decisions();
    final generation = graph.focus.generationFor(focus);
    final planEvidence = formatWarpFocusPlanTimeoutEvidence(
      page.planPage.records,
      focusGeneration: generation,
    );
    fail(
      '${_timeoutEvidence(timeout, 'decoded=${focus.videoId.value}')} '
      '${_focusEvidence(focus)} '
      '$planEvidence '
      'decisions=${_decisionEvidence(decisions.records)}',
    );
  }

  String _focusEvidence(PlaybackFocus focus) {
    final index = scenario.events.indexWhere(
      (event) => event.id == focus.videoId.value,
    );
    final state = graph.cubit.state;
    final active = state is FeedLoaded ? state.activeIndex : -1;
    final delivery = graph.focus.deliveryForEvent(focus.videoId.value);
    final stages = delivery == null
        ? const <WarpFeedPlayerStageEvidence>[]
        : graph.playerStages.attemptsFor(delivery);
    return 'waiting=long-${index.toString().padLeft(2, '0')}/'
        '${focus.sequence}, active=$active, stages=${_stageEvidence(stages)}, '
        'origin=${_requestEvidence(index)}';
  }

  String _stageEvidence(List<WarpFeedPlayerStageEvidence> stages) => stages
      .map(
        (stage) =>
            'p${stage.preparedAt.inMilliseconds}/'
            'i${stage.initializingAt?.inMilliseconds}/'
            'd${stage.initializedAt?.inMilliseconds}/'
            'f${stage.firstFrameAt?.inMilliseconds}/'
            'x${stage.failedAt?.inMilliseconds}/'
            'r${stage.releasedAt?.inMilliseconds}',
      )
      .join('|');

  String _requestEvidence(int index) {
    if (index < 0) return 'unknown';
    final id = 'long-${index.toString().padLeft(2, '0')}';
    return origin.requestsFor(id).map(_requestSummary).join('|');
  }

  String _requestSummary(ProgressiveOriginRequest request) {
    final range = request.range;
    final span = range == null ? 'whole' : '${range.start}-${range.end}';
    return '${request.method}:$span:${request.outcome.name}/'
        '${request.servedBytes}';
  }

  String _decisionEvidence(Iterable<WarpDecisionRecord> decisions) => decisions
      .toList()
      .reversed
      .take(8)
      .map(
        (record) =>
            '${record.sequence}:${record.selected?.kind}/'
            '${record.selected?.command}:${record.outcome.status}/'
            '${record.outcome.bytes}',
      )
      .join('|');
  String _timeoutEvidence(Duration timeout, String? awaiting) {
    final loaded = graph.cubit.state;
    final active = loaded is FeedLoaded
        ? '${loaded.activeIndex}:${loaded.posts[loaded.activeIndex].id.value}'
        : 'none';
    return 'Long WARP session timed out after $timeout; '
        'awaiting=${awaiting ?? 'condition'}, activePage=$active, '
        'state=${graph.cubit.state.runtimeType}, posts=$_loadedPostCount, '
        'focuses=${graph.focus.occurrences.length}, handoffs=$handoffs, '
        'players=$peakMountedPlayers, requests=${origin.requests.length}, '
        'active=${origin.activeIncompleteRequestSequences}.';
  }
}
