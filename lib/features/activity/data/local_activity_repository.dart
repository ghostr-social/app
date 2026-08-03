import 'dart:convert';

import 'package:ghostr/core/async/keyed_serial_task_queue.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/core/storage/preference_storage_guard.dart';
import 'package:ghostr/features/activity/data/activity_item_storage_mapper.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:shared_preferences/shared_preferences.dart';

class LocalActivityRepository implements AccountScopedActivityStore {
  LocalActivityRepository(
    SharedPreferences preferences, {
    required AccountStorageScope accountScope,
    ActivityItemStorageMapper mapper = const ActivityItemStorageMapper(),
  })  : _resources = _LocalActivityResources(
          preferences,
          accountScope,
          mapper,
          KeyedSerialTaskQueue(),
        ),
        _pinnedAccount = null;

  LocalActivityRepository._(this._resources, this._pinnedAccount);

  static const _key = 'ghostr.activity.items';

  final _LocalActivityResources _resources;
  final AccountStorageKey? _pinnedAccount;
  SharedPreferences get _preferences => _resources.preferences;
  AccountStorageScope get _accountScope => _resources.accountScope;
  ActivityItemStorageMapper get _mapper => _resources.mapper;
  KeyedSerialTaskQueue get _queue => _resources.queue;

  @override
  LocalActivityRepository snapshotForActiveAccount() {
    return _snapshot(_pinnedAccount ?? _accountScope.capture());
  }

  @override
  LocalActivityRepository snapshotForAccount(NostrPublicKeyHex account) {
    return _snapshot(AccountStorageKey(account));
  }

  LocalActivityRepository _snapshot(AccountStorageKey account) {
    if (_pinnedAccount == account) return this;
    return LocalActivityRepository._(_resources, account);
  }

  @override
  Future<List<ActivityItem>> load() {
    final account = _account;
    return guardPreferenceStorage(
      'Could not read local activity.',
      () => _load(account),
    );
  }

  List<ActivityItem> _load(AccountStorageKey account) {
    final raw = _preferences.getString(account.key(_key));
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
    final account = _account;
    return _queue.run(
      account,
      () => guardPreferenceStorage(
        'Could not save local activity.',
        () => _record(item, account),
      ),
    );
  }

  Future<void> _record(ActivityItem item, AccountStorageKey account) async {
    final next = <ActivityItem>[item, ..._load(account)].take(50).toList();
    final payload = next.map(_mapper.toMap).toList();
    await requirePreferenceWrite(
      'Could not save local activity.',
      () => _preferences.setString(
        account.key(_key),
        jsonEncode(payload),
      ),
    );
  }

  AccountStorageKey get _account => _pinnedAccount ?? _accountScope.capture();
}

class _LocalActivityResources {
  const _LocalActivityResources(
    this.preferences,
    this.accountScope,
    this.mapper,
    this.queue,
  );

  final SharedPreferences preferences;
  final AccountStorageScope accountScope;
  final ActivityItemStorageMapper mapper;
  final KeyedSerialTaskQueue queue;
}
