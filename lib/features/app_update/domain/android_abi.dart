enum AndroidAbi {
  arm64V8a('arm64-v8a'),
  armeabiV7a('armeabi-v7a'),
  x86_64('x86_64');

  const AndroidAbi(this.value);

  final String value;

  static AndroidAbi? tryParse(String raw) {
    for (final abi in values) {
      if (abi.value == raw) return abi;
    }
    return null;
  }
}
