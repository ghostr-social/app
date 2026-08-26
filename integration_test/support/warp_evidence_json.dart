part of 'warp_evidence_models.dart';

void _warpSchema(Map<String, Object?> json) {
  final version = _warpInt(json, 'schema_version');
  if (version != 1) throw FormatException('Unsupported WARP schema: $version');
}

Map<String, Object?> _warpObject(Object? value, String path) {
  if (value is! Map) throw FormatException('$path must be an object.');
  final result = <String, Object?>{};
  for (final entry in value.entries) {
    if (entry.key is! String) {
      throw FormatException('$path contains a non-string key.');
    }
    result[entry.key as String] = entry.value;
  }
  return result;
}

Object? _warpRequired(Map<String, Object?> json, String field) {
  if (!json.containsKey(field)) throw FormatException('Missing $field.');
  return json[field];
}

Map<String, Object?> _warpChild(Map<String, Object?> json, String field) {
  return _warpObject(_warpRequired(json, field), field);
}

List<Object?> _warpList(Map<String, Object?> json, String field) {
  final value = _warpRequired(json, field);
  if (value is! List) throw FormatException('$field must be a list.');
  return value.cast<Object?>();
}

int _warpInt(Map<String, Object?> json, String field) {
  final value = _warpRequired(json, field);
  if (value is! int) throw FormatException('$field must be an integer.');
  return value;
}

int? _warpOptionalInt(Map<String, Object?> json, String field) {
  final value = _warpRequired(json, field);
  if (value == null) return null;
  if (value is! int) throw FormatException('$field must be an integer.');
  return value;
}

bool _warpBool(Map<String, Object?> json, String field) {
  final value = _warpRequired(json, field);
  if (value is! bool) throw FormatException('$field must be a boolean.');
  return value;
}

String _warpString(Map<String, Object?> json, String field) {
  final value = _warpRequired(json, field);
  if (value is! String || value.isEmpty) {
    throw FormatException('$field must be a non-empty string.');
  }
  return value;
}

String? _warpOptionalString(Map<String, Object?> json, String field) {
  final value = _warpRequired(json, field);
  if (value == null) return null;
  if (value is! String || value.isEmpty) {
    throw FormatException('$field must be a non-empty string.');
  }
  return value;
}
