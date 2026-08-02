import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/session/domain/nostr_identity_deriver.dart';
import 'package:ndk/ndk.dart';

typedef DecodePrivateKey = String Function(String encoded);
typedef DerivePublicKey = String Function(String privateKey);
typedef EncodeNpub = String Function(String publicKey);

class NostrIdentityDerivationSteps {
  const NostrIdentityDerivationSteps({
    required this.decodePrivateKey,
    required this.derivePublicKey,
    required this.encodeNpub,
  });

  final DecodePrivateKey decodePrivateKey;
  final DerivePublicKey derivePublicKey;
  final EncodeNpub encodeNpub;
}

class NdkNostrIdentityDeriver implements NostrIdentityDeriver {
  const NdkNostrIdentityDeriver({this.steps});

  final NostrIdentityDerivationSteps? steps;

  @override
  NostrIdentity derive(AuthSecret secret) {
    try {
      return _derive(secret, steps ?? _defaultSteps);
    } on FormatException {
      throw const FormatException('Enter a valid nsec1 secret.');
    } on Object catch (error, stackTrace) {
      logBoundaryFailure(
        source: 'NdkNostrIdentityDeriver.derive',
        message: 'Nostr identity derivation failed.',
        error: error.runtimeType,
        stackTrace: stackTrace,
      );
      throw const AppFailure('Could not import the Nostr key securely.');
    }
  }

  NostrIdentity _derive(
    AuthSecret secret,
    NostrIdentityDerivationSteps activeSteps,
  ) {
    final privateKey = activeSteps.decodePrivateKey(secret.value);
    if (!_isPrivateKey(privateKey)) throw const FormatException();
    final publicKey = activeSteps.derivePublicKey(privateKey);
    return NostrIdentity.parse(
      publicKeyHex: publicKey,
      npub: activeSteps.encodeNpub(publicKey),
    );
  }

  bool _isPrivateKey(String value) {
    return value.length == 64 && RegExp(r'^[0-9a-f]+$').hasMatch(value);
  }
}

final _defaultSteps = NostrIdentityDerivationSteps(
  decodePrivateKey: Nip19.decode,
  derivePublicKey: const Bip340EventSignerFactory().derivePublicKey,
  encodeNpub: Nip19.encodePubKey,
);
