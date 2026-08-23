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
part 'ffi_player_preparation_feedback_drain.dart';

typedef RustPlayerPreparationReporter =
    Future<FfiPlayerPreparationDisposition> Function({
      required FfiPlayerPreparationReport input,
    });
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

  static const _pendingLimit = 6;
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
  final Set<BigInt> _dispatchedAttempts = {};
  final LinkedHashMap<BigInt, _FfiPlayerPreparationAttempt> _attempts =
      LinkedHashMap();
  final Map<BigInt, List<FfiPlayerPreparationReport>> _history = {};
  final Map<BigInt, Future<void>> _draining = {};
  final Set<BigInt> _noAdmissionAttempts = {};
  final Map<BigInt, Completer<void>> _retryWakes = {};
  var _closed = false;

  @override
  PlayerPreparationAttempt prepare(PlaybackAssetAuthority authority) {
    return _FfiPlayerPreparationAttempt(
      this,
      authority,
      BigInt.from(++_nextAttemptGeneration),
      _tokenFactory(),
    );
  }

  void _send(
    FfiPlayerPreparationReport report,
    _FfiPlayerPreparationAttempt source,
  ) {
    if (_closed || !_trackAttempt(report, source)) {
      source._discard();
      return;
    }
    final attempt = report.attemptGeneration;
    _pending.putIfAbsent(attempt, ListQueue.new).add(report);
    _draining[attempt] ??= _drain(attempt);
  }

  bool _trackAttempt(
    FfiPlayerPreparationReport report,
    _FfiPlayerPreparationAttempt source,
  ) {
    if (report.state != FfiPlayerPreparationState.initializing) return true;
    while (_attempts.length >= _pendingLimit) {
      final victim = _oldestEvictableAttempt();
      if (victim == null) return false;
      _discardAttempt(victim);
    }
    _attempts[report.attemptGeneration] = source;
    return true;
  }

  BigInt? _oldestEvictableAttempt() {
    for (final key in _attempts.keys) {
      if (_isEvictable(key)) return key;
    }
    return null;
  }

  bool _isEvictable(BigInt attempt) {
    if (!_dispatchedAttempts.contains(attempt)) return true;
    if (!_noAdmissionAttempts.contains(attempt)) return false;
    return _attempts[attempt]?._terminal == true;
  }

  bool _acknowledged(FfiPlayerPreparationDisposition disposition) =>
      disposition == FfiPlayerPreparationDisposition.applied ||
      disposition == FfiPlayerPreparationDisposition.duplicate;

  bool _discarded(FfiPlayerPreparationDisposition disposition) =>
      disposition == FfiPlayerPreparationDisposition.stale ||
      disposition == FfiPlayerPreparationDisposition.rejected;

  bool _isTerminal(FfiPlayerPreparationState state) =>
      state == FfiPlayerPreparationState.failed ||
      state == FfiPlayerPreparationState.released;

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
