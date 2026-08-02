abstract interface class SecretStore {
  Future<String?> read();
  Future<void> write(String value);
  Future<void> clear();
}
