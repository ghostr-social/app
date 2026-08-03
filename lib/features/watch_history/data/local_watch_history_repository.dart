import 'dart:convert';

import 'package:ghostr/core/async/keyed_serial_task_queue.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/core/storage/preference_storage_guard.dart';
import 'package:ghostr/features/watch_history/data/watch_history_entry_storage_mapper.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

class LocalWatchHistoryRepository implements WatchHistoryRepository {
  LocalWatchHistoryRepository(
    SharedPreferences preferences, {
    required AccountStorageScope accountScope,
    WatchHistoryEntryStorageMapper mapper =
        const WatchHistoryEntryStorageMapper(),
  })  : _resources = _LocalWatchHistoryResources(
          preferences,
          accountScope,
          mapper,
          KeyedSerialTaskQueue(),
        ),
        _pinnedAccount = null;

  LocalWatchHistoryRepository._(this._resources, this._pinnedAccount);

  static const _key = 'ghostr.history.watched';
  static const _capacity = 2000;

  final _LocalWatchHistoryResources _resources;
  final AccountStorageKey? _pinnedAccount;
  SharedPreferences get _preferences => _resources.preferences;
  AccountStorageScope get _accountScope => _resources.accountScope;
  WatchHistoryEntryStorageMapper get _mapper => _resources.mapper;
  KeyedSerialTaskQueue get _queue => _resources.queue;

  @override
  LocalWatchHistoryRepository snapshotForActiveAccount() {
    final account = _pinnedAccount ?? _accountScope.capture();
    if (_pinnedAccount == account) return this;
    return LocalWatchHistoryRepository._(_resources, account);
  }

  // Reads share the write queue so a load started right after a record
  // cannot observe the history without it.
  @override
  Future<List<WatchHistoryEntry>> load() {
    final account = _account;
    return _queue.run(
      account,
      () => guardPreferenceStorage(
        'Could not read watch history.',
        () => _load(account),
      ),
    );
  }

  List<WatchHistoryEntry> _load(AccountStorageKey account) {
    final raw = _preferences.getString(account.key(_key));
    if (raw == null || raw.isEmpty) {
      return const <WatchHistoryEntry>[];
    }
    final decoded = jsonDecode(raw) as List<dynamic>;
    return decoded
        .map((entry) => _mapper.fromMap(entry as Map<String, dynamic>))
        .toList()
      ..sort((left, right) => right.watchedAt.compareTo(left.watchedAt));
  }

  @override
  Future<void> record(WatchHistoryEntry entry) {
    final account = _account;
    return _queue.run(
      account,
      () => guardPreferenceStorage(
        'Could not save watch history.',
        () => _record(entry, account),
      ),
    );
  }

  Future<void> _record(
    WatchHistoryEntry entry,
    AccountStorageKey account,
  ) async {
    final existing =
        _load(account).where((item) => item.videoId != entry.videoId);
    final next =
        <WatchHistoryEntry>[entry, ...existing].take(_capacity).toList();
    final payload = next.map(_mapper.toMap).toList();
    await requirePreferenceWrite(
      'Could not save watch history.',
      () => _preferences.setString(
        account.key(_key),
        jsonEncode(payload),
      ),
    );
  }

  @override
  Future<void> clear() {
    final account = _account;
    return _queue.run(
      account,
      () => guardPreferenceStorage(
        'Could not clear watch history.',
        () => _clear(account),
      ),
    );
  }

  Future<void> _clear(AccountStorageKey account) async {
    await requirePreferenceWrite(
      'Could not clear watch history.',
      () => _preferences.remove(account.key(_key)),
    );
  }

  AccountStorageKey get _account => _pinnedAccount ?? _accountScope.capture();
}

class _LocalWatchHistoryResources {
  const _LocalWatchHistoryResources(
    this.preferences,
    this.accountScope,
    this.mapper,
    this.queue,
  );

  final SharedPreferences preferences;
  final AccountStorageScope accountScope;
  final WatchHistoryEntryStorageMapper mapper;
  final KeyedSerialTaskQueue queue;
}
