import 'package:ghostr/core/storage/secret_store.dart';

class MemorySecretStore implements SecretStore {
  MemorySecretStore({this.readError, this.writeError, this.clearError});

  String? value;
  final Object? readError;
  final Object? writeError;
  final Object? clearError;

  @override
  Future<void> clear() async {
    if (clearError != null) throw clearError!;
    value = null;
  }

  @override
  Future<String?> read() async {
    if (readError != null) throw readError!;
    return value;
  }

  @override
  Future<void> write(String newValue) async {
    if (writeError != null) throw writeError!;
    value = newValue;
  }
}
