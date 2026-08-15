import 'package:ghostr/core/async/keyed_serial_task_queue.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/core/storage/preference_storage_guard.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/watch_history/data/legacy_watch_history_decoder.dart';
import 'package:ghostr/features/watch_history/data/watch_history_sembast_store.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_repository.dart';
import 'package:sembast/sembast.dart';
import 'package:shared_preferences/shared_preferences.dart';

final class LocalWatchHistoryRepository implements WatchHistoryRepository {
  LocalWatchHistoryRepository(
    SharedPreferences preferences, {
    required Database database,
    required AccountStorageScope accountScope,
    LegacyWatchHistoryDecoder decoder = const LegacyWatchHistoryDecoder(),
  }) : _resources = _LocalWatchHistoryResources(
         preferences,
         accountScope,
         decoder,
         WatchHistorySembastStore(database),
         KeyedSerialTaskQueue(),
       ),
       _pinnedAccount = null;

  LocalWatchHistoryRepository._(this._resources, this._pinnedAccount);

  static const _legacyKey = 'ghostr.history.watched';
  static const _closedFailure = AppFailure('Watch history is closed.');

  final _LocalWatchHistoryResources _resources;
  final AccountStorageKey? _pinnedAccount;

  @override
  LocalWatchHistoryRepository snapshotForActiveAccount() {
    final account = _pinnedAccount ?? _resources.accountScope.capture();
    if (_pinnedAccount == account) return this;
    return LocalWatchHistoryRepository._(_resources, account);
  }

  @override
  Future<List<WatchHistoryEntry>> load() {
    final account = _account;
    return _run(account, 'Could not read watch history.', () async {
      await _prepare(account);
      return _resources.store.loadRecent(account);
    });
  }

  @override
  Future<List<VideoPost>> filterUnwatched(List<VideoPost> posts) {
    final account = _account;
    return _run(account, 'Could not verify watched videos.', () async {
      await _prepare(account);
      return _resources.store.filterUnwatched(account, posts);
    });
  }

  @override
  Future<void> record(WatchHistoryEntry entry) {
    if (_resources.isClosing) return Future<void>.error(_closedFailure);
    final account = _account;
    return _resources.queue.run(account, () => _record(account, entry));
  }

  Future<void> _record(
    AccountStorageKey account,
    WatchHistoryEntry entry,
  ) async {
    try {
      await _guard('Could not save watch history.', () async {
        await _prepare(account);
        await _resources.store.record(account, entry);
      });
    } on Object {
      _resources.unsafeAccounts.add(account);
      rethrow;
    }
  }

  @override
  Future<void> clear() {
    final account = _account;
    return _run(
      account,
      'Could not clear watch history.',
      () => _clear(account),
    );
  }

  Future<void> _clear(AccountStorageKey account) async {
    await _resources.store.clear(account);
    _resources.unsafeAccounts.remove(account);
    await requirePreferenceWrite(
      'Could not clear watch history.',
      () => _resources.preferences.remove(account.key(_legacyKey)),
    );
  }

  Future<void> close() {
    final existing = _resources.closeFuture;
    if (existing != null) return existing;
    _resources.isClosing = true;
    final closing = _drainAndClose();
    _resources.closeFuture = closing;
    return closing;
  }

  Future<void> _drainAndClose() async {
    await _resources.queue.drain();
    await _resources.store.close();
  }

  Future<void> _prepare(AccountStorageKey account) async {
    if (_resources.unsafeAccounts.contains(account)) {
      throw const AppFailure('Watch history is not safely persisted.');
    }
    if (await _resources.store.isMigrated(account)) return;
    final raw = _resources.preferences.getString(account.key(_legacyKey));
    final migration = _resources.decoder.decode(raw);
    await _resources.store.migrate(account, migration);
  }

  Future<T> _run<T>(
    AccountStorageKey account,
    String message,
    Future<T> Function() operation,
  ) {
    if (_resources.isClosing) return Future<T>.error(_closedFailure);
    return _resources.queue.run(account, () => _guard(message, operation));
  }

  Future<T> _guard<T>(String message, Future<T> Function() operation) async {
    try {
      return await operation();
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'ghostr.storage.watchHistory',
        message: message,
        error: error,
        stackTrace: stackTrace,
      );
    }
  }

  AccountStorageKey get _account {
    return _pinnedAccount ?? _resources.accountScope.capture();
  }
}

final class _LocalWatchHistoryResources {
  _LocalWatchHistoryResources(
    this.preferences,
    this.accountScope,
    this.decoder,
    this.store,
    this.queue,
  );

  final SharedPreferences preferences;
  final AccountStorageScope accountScope;
  final LegacyWatchHistoryDecoder decoder;
  final WatchHistorySembastStore store;
  final KeyedSerialTaskQueue queue;
  final Set<AccountStorageKey> unsafeAccounts = <AccountStorageKey>{};
  bool isClosing = false;
  Future<void>? closeFuture;
}
