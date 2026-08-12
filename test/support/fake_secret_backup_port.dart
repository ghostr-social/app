import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/secret_backup_port.dart';

final class FakeSecretBackupPort implements SecretBackupPort {
  AuthSecret? copiedSecret;

  @override
  Future<void> copy(AuthSecret secret) async {
    copiedSecret = secret;
  }
}
