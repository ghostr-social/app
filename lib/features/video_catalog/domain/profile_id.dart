extension type const ProfileId._(String value) implements String {
  factory ProfileId.parse(String raw) {
    final value = raw.trim();
    if (value.isEmpty) {
      throw const FormatException('Profile identifiers cannot be empty.');
    }
    return ProfileId._(value);
  }
}
