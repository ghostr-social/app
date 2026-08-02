import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';

abstract interface class NostrIdentityDeriver {
  NostrIdentity derive(AuthSecret secret);
}
