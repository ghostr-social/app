import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/platform/nostr/build_ndk.dart';
import 'package:ndk/ndk.dart';

final class WarpFeedNostrAccount {
  const WarpFeedNostrAccount._(this.ndk, this.secret, this.identity);

  factory WarpFeedNostrAccount.create() {
    final keys = const Bip340EventSignerFactory().generateKeyPair();
    return WarpFeedNostrAccount._(
      buildNdk(),
      AuthSecret.parse(Nip19.encodePrivateKey(keys.$1)),
      NostrIdentity.parse(
        publicKeyHex: keys.$2,
        npub: Nip19.encodePubKey(keys.$2),
      ),
    );
  }

  final Ndk ndk;
  final AuthSecret secret;
  final NostrIdentity identity;

  Future<void> activate(ProductionNostrServices services) {
    return services.adapters.session.activate(secret, identity);
  }
}
