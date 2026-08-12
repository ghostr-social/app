final class AndroidVersionCode implements Comparable<AndroidVersionCode> {
  factory AndroidVersionCode(int value) {
    if (value < 1 || value > maximum) {
      throw ArgumentError.value(value, 'value', 'Must be a valid versionCode.');
    }
    return AndroidVersionCode._(value);
  }

  const AndroidVersionCode._(this.value);

  static const maximum = 2100000000;

  final int value;

  @override
  int compareTo(AndroidVersionCode other) => value.compareTo(other.value);
}
