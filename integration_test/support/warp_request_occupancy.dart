final class WarpRequestOccupancy {
  const WarpRequestOccupancy._(this.total, this.maximumPerOrigin);

  factory WarpRequestOccupancy.fromJson(Map<String, Object?> json) {
    final total = _count(json['total']);
    final authorities = json['authorities'];
    if (authorities is! Map<String, Object?> || _count(json['invalid']) != 0) {
      throw const FormatException(
        'Native request authorities are missing or invalid.',
      );
    }
    var sum = 0;
    var peak = 0;
    for (final value in authorities.values) {
      final count = _count(value);
      sum += count;
      if (count > peak) peak = count;
    }
    if (sum != total) {
      throw const FormatException('Native request totals disagree.');
    }
    return WarpRequestOccupancy._(total, peak);
  }

  final int total;
  final int maximumPerOrigin;
  bool get withinCoreBounds => total <= 2 && maximumPerOrigin <= 1;
}

int _count(Object? value) {
  if (value is int && value >= 0) return value;
  throw const FormatException(
    'Native request counts must be nonnegative integers.',
  );
}
