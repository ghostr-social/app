class WatchHistoryLedgerBucketMapper {
  const WatchHistoryLedgerBucketMapper();

  Map<String, Object?> toMap(Set<String> fingerprints) {
    final sorted = fingerprints.toList()..sort();
    return <String, Object?>{'schema': 1, 'fingerprints': sorted};
  }

  Set<String> fromMap(Map<String, Object?> map, {required String bucket}) {
    final values = <String>{};
    for (final value in _storedFingerprints(map)) {
      values.add(_validatedFingerprint(value, bucket));
    }
    return values;
  }

  List<Object?> _storedFingerprints(Map<String, Object?> map) {
    if (map['schema'] != 1) throw _invalidBucket;
    final stored = map['fingerprints'];
    if (stored is! List) throw _invalidBucket;
    return stored.cast<Object?>();
  }

  String _validatedFingerprint(Object? value, String bucket) {
    if (value is! String) throw _invalidFingerprint;
    if (!_isFingerprint(value, bucket)) throw _invalidFingerprint;
    return value;
  }

  static const _invalidBucket = FormatException(
    'Watched-video ledger bucket is invalid.',
  );
  static const _invalidFingerprint = FormatException(
    'Watched-video fingerprint is invalid.',
  );

  bool _isFingerprint(String value, String bucket) {
    return value.startsWith(bucket) &&
        RegExp(r'^[0-9a-f]{32}$').hasMatch(value);
  }
}
