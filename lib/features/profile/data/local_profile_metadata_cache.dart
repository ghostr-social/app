import 'dart:convert';

import 'package:ghostr/core/storage/preference_storage_guard.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:shared_preferences/shared_preferences.dart';

final class LocalProfileMetadataCache {
  const LocalProfileMetadataCache(this._preferences);

  static const _namespace = 'ghostr.profile.metadata';
  final SharedPreferences _preferences;

  Future<ProfileSummary?> read(ProfileId id) async {
    return (await readSnapshot(id))?.profile;
  }

  Future<CachedProfileMetadata?> readSnapshot(ProfileId id) {
    return guardPreferenceStorage('Could not read the cached profile.', () {
      final raw = _preferences.getString(_key(id));
      if (raw == null || raw.isEmpty) return null;
      final payload = jsonDecode(raw);
      if (payload is! Map<String, dynamic>) throw const FormatException();
      return CachedProfileMetadata(
        _summary(id, payload),
        observedAt: _observedAt(payload),
      );
    });
  }

  Future<void> write(ProfileSummary summary, {int observedAt = 0}) {
    if (observedAt < 0) throw const FormatException('Invalid cache timestamp.');
    return requirePreferenceWrite(
      'Could not cache the profile.',
      () => _preferences.setString(
        _key(summary.id),
        jsonEncode({
          'displayName': summary.displayName,
          'handle': summary.handle,
          'avatarUrl': summary.avatarUrl,
          'observedAt': observedAt,
        }),
      ),
    );
  }

  ProfileSummary _summary(ProfileId id, Map<String, dynamic> payload) {
    return ProfileSummary(
      id: id,
      displayName: _required(payload, 'displayName'),
      handle: _required(payload, 'handle'),
      avatarUrl: _optional(payload, 'avatarUrl'),
    );
  }

  String _required(Map<String, dynamic> payload, String key) {
    return _optional(payload, key) ?? (throw const FormatException());
  }

  String? _optional(Map<String, dynamic> payload, String key) {
    final value = payload[key];
    if (value == null || value is String) return value as String?;
    throw const FormatException();
  }

  int _observedAt(Map<String, dynamic> payload) {
    final value = payload['observedAt'];
    if (value == null) return 0;
    if (value is int && value >= 0) return value;
    throw const FormatException();
  }

  String _key(ProfileId id) => '$_namespace.${id.value}';
}

final class CachedProfileMetadata {
  const CachedProfileMetadata(this.profile, {required this.observedAt});

  final ProfileSummary profile;
  final int observedAt;
}
