import 'dart:convert';

import 'package:ghostr/core/storage/preference_storage_guard.dart';
import 'package:ghostr/features/activity/data/activity_item_storage_mapper.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:shared_preferences/shared_preferences.dart';

class LocalActivityRepository implements ActivityRepository {
  LocalActivityRepository(
    this._preferences, {
    ActivityItemStorageMapper mapper = const ActivityItemStorageMapper(),
  }) : _mapper = mapper;

  static const _key = 'ghostr.activity.items';

  final SharedPreferences _preferences;
  final ActivityItemStorageMapper _mapper;

  @override
  Future<List<ActivityItem>> load() {
    return guardPreferenceStorage(
      'Could not read local activity.',
      _load,
    );
  }

  List<ActivityItem> _load() {
    final raw = _preferences.getString(_key);
    if (raw == null || raw.isEmpty) {
      return const <ActivityItem>[];
    }
    final decoded = jsonDecode(raw) as List<dynamic>;
    return decoded
        .map((item) => _mapper.fromMap(item as Map<String, dynamic>))
        .toList()
      ..sort((left, right) => right.occurredAt.compareTo(left.occurredAt));
  }

  @override
  Future<void> record(ActivityItem item) {
    return guardPreferenceStorage(
      'Could not save local activity.',
      () => _record(item),
    );
  }

  Future<void> _record(ActivityItem item) async {
    final next = <ActivityItem>[item, ...await load()].take(50).toList();
    final payload = next.map(_mapper.toMap).toList();
    await requirePreferenceWrite(
      'Could not save local activity.',
      () => _preferences.setString(_key, jsonEncode(payload)),
    );
  }
}
