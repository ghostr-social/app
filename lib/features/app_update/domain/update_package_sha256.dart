final class UpdatePackageSha256 {
  factory UpdatePackageSha256.parse(String raw) {
    final digest = tryParse(raw);
    if (digest == null) {
      throw const FormatException('A lowercase SHA-256 digest is required.');
    }
    return digest;
  }

  const UpdatePackageSha256._(this.value);

  static final _pattern = RegExp(r'^[0-9a-f]{64}$');

  final String value;

  static UpdatePackageSha256? tryParse(String raw) {
    return _pattern.hasMatch(raw) ? UpdatePackageSha256._(raw) : null;
  }
}
