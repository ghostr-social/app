import 'package:ghostr/features/session/domain/nostr_session_port.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';

class FakeNostrSessionPort implements NostrSessionPort {
  int activationCount = 0;
  int deactivationCount = 0;

  @override
  void activate(AuthSecret secret, NostrIdentity identity) {
    activationCount += 1;
  }

  @override
  void deactivate() {
    deactivationCount += 1;
  }
}
