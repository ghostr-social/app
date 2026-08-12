import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/storage/secret_store.dart';

class SecureSecretStore implements SecretStore {
  SecureSecretStore(this._storage, {String storageKey = 'ghostr.viewer.secret'})
    : _storageKey = storageKey;

  final FlutterSecureStorage _storage;
  final String _storageKey;

  @override
  Future<void> clear() => _guard(() => _storage.delete(key: _storageKey));

  @override
  Future<String?> read() => _guard(() => _storage.read(key: _storageKey));

  @override
  Future<void> write(String value) {
    return _guard(() => _storage.write(key: _storageKey, value: value));
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
