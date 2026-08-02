import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';

abstract interface class NostrSessionPort {
  void activate(AuthSecret secret, NostrIdentity identity);

  void deactivate();
}
