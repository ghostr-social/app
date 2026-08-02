import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/session/domain/nostr_session_port.dart';
import 'package:ndk/ndk.dart';

class NdkNostrSession implements NostrSessionPort {
  const NdkNostrSession(this._ndk);

  final Ndk _ndk;

  @override
  void activate(AuthSecret secret, NostrIdentity identity) {
    _guard('Could not activate the Nostr account.', () {
      if (_ndk.accounts.isLoggedIn) _ndk.accounts.logout();
      _ndk.accounts.loginPrivateKey(
        pubkey: identity.publicKeyHex,
        privkey: Nip19.decode(secret.value),
      );
    });
  }

  @override
  void deactivate() {
    _guard('Could not deactivate the Nostr account.', () {
      if (_ndk.accounts.isLoggedIn) _ndk.accounts.logout();
    });
  }

  T _guard<T>(String message, T Function() operation) {
    try {
      return operation();
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'ghostr.nostr.session',
        message: message,
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}
