final class AppVersion implements Comparable<AppVersion> {
  const AppVersion._(this.value, this.major, this.minor, this.patch);

  static final _pattern = RegExp(
    r'^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$',
  );

  final String value;
  final int major;
  final int minor;
  final int patch;

  factory AppVersion.parse(String value) {
    final parsed = tryParse(value);
    if (parsed == null) throw const FormatException('Invalid app version.');
    return parsed;
  }

  static AppVersion? tryParse(String value) {
    final match = _pattern.firstMatch(value);
    if (match == null) return null;
    try {
      return AppVersion._(
        value,
        int.parse(match[1]!),
        int.parse(match[2]!),
        int.parse(match[3]!),
      );
    } on FormatException {
      return null;
    }
  }

  @override
  int compareTo(AppVersion other) {
    final majorOrder = major.compareTo(other.major);
    if (majorOrder != 0) return majorOrder;
    final minorOrder = minor.compareTo(other.minor);
    return minorOrder != 0 ? minorOrder : patch.compareTo(other.patch);
  }

  @override
  bool operator ==(Object other) {
    return other is AppVersion && value == other.value;
  }

  @override
  int get hashCode => value.hashCode;

  @override
  String toString() => value;
}
