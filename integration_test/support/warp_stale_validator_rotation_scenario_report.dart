part of 'warp_stale_validator_rotation_scenario.dart';

extension _WarpValidatorRotationReport on _WarpValidatorRotationDriver {
  void _report(
    PlaybackFocus replacement,
    ({Duration before, Duration after}) advance,
    Uint8List bytes,
  ) {
    debugPrint(
      'WARP_VALIDATOR_ROTATION focus=${replacement.videoId.value} '
      'bytes=${bytes.length} hash=${sha256.convert(bytes)} '
      'advance_ms=${(advance.after - advance.before).inMilliseconds} '
      'redirects=${fixture.redirectTargets.map((item) => item.path).join(",")} '
      'requests=${fixture.totalRequestCount} peak='
      '${fixture.maximumConcurrentRequests} controllers='
      '$peakControllerCapacity players=$peakMountedPlayers',
    );
  }
}
