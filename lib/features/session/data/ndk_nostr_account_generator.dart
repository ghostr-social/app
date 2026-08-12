import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/generated_nostr_account.dart';
import 'package:ghostr/features/session/domain/nostr_account_generator.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ndk/ndk.dart';

typedef GeneratePrivateKey = String Function();
typedef TransformKey = String Function(String value);

final class NostrAccountGenerationSteps {
  const NostrAccountGenerationSteps({
    required this.generatePrivateKey,
    required this.derivePublicKey,
    required this.encodeNsec,
    required this.encodeNpub,
  });

  final GeneratePrivateKey generatePrivateKey;
  final TransformKey derivePublicKey;
  final TransformKey encodeNsec;
  final TransformKey encodeNpub;
}

final class NdkNostrAccountGenerator implements NostrAccountGenerator {
  const NdkNostrAccountGenerator({this.steps});

  final NostrAccountGenerationSteps? steps;

  @override
  GeneratedNostrAccount generate() {
    final active = steps ?? _defaultSteps;
    final privateKey = active.generatePrivateKey();
    final publicKey = active.derivePublicKey(privateKey);
    return GeneratedNostrAccount(
      secret: AuthSecret.parse(active.encodeNsec(privateKey)),
      identity: NostrIdentity.parse(
        publicKeyHex: publicKey,
        npub: active.encodeNpub(publicKey),
      ),
    );
  }
}

final _factory = const Bip340EventSignerFactory();
final _defaultSteps = NostrAccountGenerationSteps(
  generatePrivateKey: () => _factory.generateKeyPair().$1,
  derivePublicKey: _factory.derivePublicKey,
  encodeNsec: Nip19.encodePrivateKey,
  encodeNpub: Nip19.encodePubKey,
);
