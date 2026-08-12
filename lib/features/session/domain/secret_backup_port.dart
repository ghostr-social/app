import 'package:ghostr/features/session/domain/auth_secret.dart';

abstract interface class SecretBackupPort {
  Future<void> copy(AuthSecret secret);
}
