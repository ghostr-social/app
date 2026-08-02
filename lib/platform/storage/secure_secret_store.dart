import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/storage/secret_store.dart';

class SecureSecretStore implements SecretStore {
  SecureSecretStore(this._storage);

  static const _secretKey = 'ghostr.viewer.secret';

  final FlutterSecureStorage _storage;

  @override
  Future<void> clear() => _guard(() => _storage.delete(key: _secretKey));

  @override
  Future<String?> read() => _guard(() => _storage.read(key: _secretKey));

  @override
  Future<void> write(String value) {
    return _guard(() => _storage.write(key: _secretKey, value: value));
  }

  Future<T> _guard<T>(Future<T> Function() operation) async {
    try {
      return await operation();
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'ghostr.storage.secure',
        message: 'Secure storage is unavailable.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}
