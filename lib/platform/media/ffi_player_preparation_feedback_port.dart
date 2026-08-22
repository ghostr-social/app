import 'dart:async';
import 'dart:collection';
import 'dart:convert';
import 'dart:developer' as developer;
import 'dart:math';

import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/features/video_inventory/domain/player_preparation_feedback_port.dart';
import 'package:ghostr/platform/media/video_player_capability_generation.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';
import 'package:ghostr/src/rust/api/player_preparation_control.dart';

part 'ffi_player_preparation_attempt.dart';

typedef RustPlayerPreparationReporter =
    Future<void> Function({required FfiPlayerPreparationReport input});
typedef PlayerPreparationClock = int Function();
typedef PlayerPreparationTokenFactory =
    PlayerPreparationAttemptToken Function();

final class FfiPlayerPreparationFeedbackPort
    implements PlayerPreparationFeedbackPort {
  FfiPlayerPreparationFeedbackPort({
    RustPlayerPreparationReporter reportPreparation =
        ffiReportPlayerPreparation,
    BigInt? playerCapabilityGeneration,
    BigInt? clientEpoch,
    PlayerPreparationClock monotonicMicros = _defaultMonotonicMicros,
    PlayerPreparationTokenFactory tokenFactory = _newAttemptToken,
  }) : _reportPreparation = reportPreparation,
       _playerCapabilityGeneration =
           playerCapabilityGeneration ??
           currentVideoPlayerCapabilityGeneration(),
       _clientEpoch = clientEpoch ?? _newClientEpoch(),
       _monotonicMicros = monotonicMicros,
       _tokenFactory = tokenFactory;

  static const _pendingLimit = 3;
  static final _clock = Stopwatch()..start();
  static BigInt _lastClientEpoch = BigInt.zero;
  static var _nextAttemptGeneration = 0;

  final RustPlayerPreparationReporter _reportPreparation;
  final BigInt _playerCapabilityGeneration;
  final BigInt _clientEpoch;
  final PlayerPreparationClock _monotonicMicros;
  final PlayerPreparationTokenFactory _tokenFactory;
  final LinkedHashMap<BigInt, ListQueue<FfiPlayerPreparationReport>> _pending =
      LinkedHashMap();
  Future<void>? _draining;

  @override
  PlayerPreparationAttempt prepare(PlaybackAssetAuthority authority) {
    return _FfiPlayerPreparationAttempt(
      this,
      authority,
      BigInt.from(++_nextAttemptGeneration),
      _tokenFactory(),
    );
  }

  void _send(FfiPlayerPreparationReport report) {
    final queue = _pending.putIfAbsent(report.attemptGeneration, ListQueue.new);
    queue.add(report);
    while (_pending.length > _pendingLimit) {
      _pending.remove(_pending.keys.first);
    }
    _draining ??= _drain();
  }

  Future<void> _drain() async {
    while (_pending.isNotEmpty) {
      final key = _pending.keys.first;
      final queue = _pending[key]!;
      final report = queue.removeFirst();
      if (queue.isEmpty) _pending.remove(key);
      try {
        await _reportPreparation(input: report);
      } on Object catch (error, stackTrace) {
        developer.log(
          'Player preparation evidence did not reach WARP.',
          name: 'ghostr.video.preparation',
          error: error,
          stackTrace: stackTrace,
        );
      }
    }
    _draining = null;
  }

  FfiPlayerPreparationReport _report(
    PlaybackAssetAuthority authority,
    BigInt attempt,
    BigInt sequence,
    FfiPlayerPreparationState state, {
    String? failureKind,
  }) {
    return FfiPlayerPreparationReport(
      postId: authority.deliveryId.value,
      representationId: authority.representationId.value,
      assetId: authority.assetId.value,
      playerCapabilityGeneration: _playerCapabilityGeneration,
      clientEpoch: _clientEpoch,
      attemptGeneration: attempt,
      sequence: sequence,
      state: state,
      failureKind: failureKind,
      observedMonotonicUs: BigInt.from(_monotonicMicros()),
    );
  }

  static int _defaultMonotonicMicros() => _clock.elapsedMicroseconds + 1;

  static BigInt _newClientEpoch() {
    final observed = BigInt.from(DateTime.now().microsecondsSinceEpoch);
    _lastClientEpoch = observed > _lastClientEpoch
        ? observed
        : _lastClientEpoch + BigInt.one;
    return _lastClientEpoch;
  }

  static PlayerPreparationAttemptToken _newAttemptToken() {
    final random = Random.secure();
    final bytes = List<int>.generate(16, (_) => random.nextInt(256));
    final raw = base64Url.encode(bytes).replaceAll('=', '');
    return PlayerPreparationAttemptToken.parse(raw);
  }
}
