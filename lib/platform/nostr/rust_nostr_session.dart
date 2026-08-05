import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/session/domain/nostr_session_port.dart';

typedef RustNostrSessionReset = Future<void> Function(
  NostrPublicKeyHex? expectedAccount,
);

const nostrSessionDivergenceFailure = AppFailure(
  'The local signer and Rust Nostr session may differ; restart the app.',
);

/// Clears account-scoped Rust state before changing the local signer.
final class RustNostrSession implements NostrSessionPort {
  RustNostrSession({
    required NostrSessionPort local,
    required RustNostrSessionReset reset,
  })  : _local = local,
        _reset = reset;

  final NostrSessionPort _local;
  final RustNostrSessionReset _reset;
  NostrPublicKeyHex? _activeAccount;

  @override
  Future<void> activate(AuthSecret secret, NostrIdentity identity) async {
    final previousAccount = _activeAccount;
    await _resetEngine(identity.publicKeyHex);
    try {
      await _local.activate(secret, identity);
    } on Object catch (error, stackTrace) {
      final restored = await _restoreEngine(previousAccount);
      if (!restored) throw nostrSessionDivergenceFailure;
      Error.throwWithStackTrace(error, stackTrace);
    }
    _activeAccount = identity.publicKeyHex;
  }

  @override
  Future<void> deactivate() async {
    final previousAccount = _activeAccount;
    await _resetEngine(null);
    try {
      await _local.deactivate();
    } on Object catch (error, stackTrace) {
      final restored = await _restoreEngine(previousAccount);
      if (!restored) throw nostrSessionDivergenceFailure;
      Error.throwWithStackTrace(error, stackTrace);
    }
    _activeAccount = null;
  }

  Future<bool> _restoreEngine(NostrPublicKeyHex? account) async {
    try {
      await _reset(account);
      return true;
    } on Object catch (error, stackTrace) {
      logBoundaryFailure(
        source: 'ghostr.nostr.session.rust',
        message: 'Could not restore the previous Nostr engine session.',
        error: error,
        stackTrace: stackTrace,
      );
      return false;
    }
  }

  Future<void> _resetEngine(NostrPublicKeyHex? expectedAccount) async {
    try {
      await _reset(expectedAccount);
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'ghostr.nostr.session.rust',
        message: 'Could not reset the Nostr engine session.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}
