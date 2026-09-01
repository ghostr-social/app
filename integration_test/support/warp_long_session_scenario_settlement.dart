part of 'warp_long_session_scenario.dart';

extension _WarpLongSessionSettlement on _WarpLongSessionDriver {
  Future<PlaybackFocus> _waitForDecodedOrRescue(PlaybackFocus intended) async {
    PlaybackFocus? rescue;
    await _wait(() {
      if (_hasDecodedPlayback(intended)) return true;
      rescue = _transportRescueAfter(intended.sequence);
      return rescue != null;
    }, awaiting: 'decodedOrRescue=${intended.videoId.value}');
    final settled = rescue ?? intended;
    await _waitForDecodedPlayback(settled);
    return settled;
  }

  PlaybackFocus? _transportRescueAfter(int sequence) => graph.focus.occurrences
      .where(
        (focus) =>
            focus.sequence > sequence &&
            focus.cause == FeedFocusCause.transportRescue,
      )
      .firstOrNull;

  int _recordSettledFocus(
    List<String> expected,
    int cursor,
    PlaybackFocus intended,
    PlaybackFocus settled,
  ) {
    final selected = expected.indexOf(settled.videoId.value, cursor);
    expect(selected, greaterThanOrEqualTo(cursor));
    visited.add(intended.videoId.value);
    visited.add(settled.videoId.value);
    final rescued = settled.sequence != intended.sequence;
    handoffs += rescued ? 2 : 1;
    decodedHandoffs += 1;
    if (rescued) transportRescues += 1;
    return selected + 1;
  }
}
