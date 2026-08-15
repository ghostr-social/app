import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/watch_history/data/legacy_watch_history_decoder.dart';
import 'package:ghostr/features/watch_history/data/watch_history_candidate_filter.dart';
import 'package:ghostr/features/watch_history/data/watch_history_entry_storage_mapper.dart';
import 'package:ghostr/features/watch_history/data/watch_history_ledger_bucket_mapper.dart';
import 'package:ghostr/features/watch_history/data/watch_history_migration_marker.dart';
import 'package:ghostr/features/watch_history/domain/video_watch_fingerprints.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:sembast/sembast.dart';

final class WatchHistorySembastStore {
  WatchHistorySembastStore(
    this._database, {
    WatchHistoryEntryStorageMapper entryMapper =
        const WatchHistoryEntryStorageMapper(),
    WatchHistoryLedgerBucketMapper bucketMapper =
        const WatchHistoryLedgerBucketMapper(),
  }) : _entryMapper = entryMapper,
       _bucketMapper = bucketMapper;

  static const _recentCapacity = 2000;
  static const _bucketPrefixLength = 3;
  static const _migration = WatchHistoryMigrationStore();
  static const _filter = WatchHistoryCandidateFilter();

  final Database _database;
  final WatchHistoryEntryStorageMapper _entryMapper;
  final WatchHistoryLedgerBucketMapper _bucketMapper;
  Future<bool> isMigrated(AccountStorageKey account) async =>
      await _migration.load(_database, account) != null;

  Future<void> migrate(
    AccountStorageKey account,
    LegacyWatchHistoryMigration migration,
  ) {
    return _database.transaction((transaction) async {
      if (await _migration.load(transaction, account) != null) return;
      await _mergeFingerprints(transaction, account, migration.identities);
      for (final entry in migration.recentEntries) {
        await _putRecent(transaction, account, entry);
      }
      await _trimRecent(transaction, account);
      await _migration.put(
        transaction,
        account,
        WatchHistoryMigrationMarker.migrated(
          migration.ordinaryPublishedThrough,
        ),
      );
    });
  }

  Future<List<WatchHistoryEntry>> loadRecent(AccountStorageKey account) async {
    final snapshots = await _recent(account).find(
      _database,
      finder: Finder(
        sortOrders: [SortOrder<int>('watchedAtMs', false)],
        limit: _recentCapacity,
      ),
    );
    return List<WatchHistoryEntry>.unmodifiable(
      snapshots.map((snapshot) => _entryMapper.fromMap(snapshot.value)),
    );
  }

  Future<List<VideoPost>> filterUnwatched(
    AccountStorageKey account,
    List<VideoPost> posts,
  ) async {
    if (posts.isEmpty) return const <VideoPost>[];
    final marker = await _migration.require(_database, account);
    final candidates = posts.map(VideoWatchFingerprints.fromPost).toList();
    final stored = await _loadFingerprints(account, candidates);
    return _filter.apply(
      posts: posts,
      candidates: candidates,
      stored: stored,
      publishedThrough: marker.ordinaryPublishedThrough,
    );
  }

  Future<void> record(AccountStorageKey account, WatchHistoryEntry entry) {
    return _database.transaction((transaction) async {
      await _mergeFingerprints(transaction, account, [
        VideoWatchFingerprints.fromEntry(entry),
      ]);
      await _putRecent(transaction, account, entry);
      await _trimRecent(transaction, account);
    });
  }

  Future<void> clear(AccountStorageKey account) {
    return _database.transaction((transaction) async {
      await _ledger(account).delete(transaction);
      await _recent(account).delete(transaction);
      await _migration.put(
        transaction,
        account,
        const WatchHistoryMigrationMarker.cleared(),
      );
    });
  }

  Future<void> close() => _database.close();

  StoreRef<String, Map<String, Object?>> _ledger(AccountStorageKey account) =>
      stringMapStoreFactory.store('watch_ledger_v1_${account.account}');

  StoreRef<String, Map<String, Object?>> _recent(AccountStorageKey account) =>
      stringMapStoreFactory.store('watch_recent_v1_${account.account}');

  Future<Set<String>> _loadFingerprints(
    AccountStorageKey account,
    List<VideoWatchFingerprints> candidates,
  ) async {
    final buckets = <String>{
      for (final identity in candidates)
        for (final value in identity.values) _bucket(value),
    };
    final snapshots = await _ledger(
      account,
    ).records(buckets).getSnapshots(_database);
    final stored = <String>{};
    for (final snapshot in snapshots.nonNulls) {
      stored.addAll(
        _bucketMapper.fromMap(snapshot.value, bucket: snapshot.key),
      );
    }
    return stored;
  }

  Future<void> _mergeFingerprints(
    DatabaseClient client,
    AccountStorageKey account,
    Iterable<VideoWatchFingerprints> identities,
  ) async {
    final additions = <String, Set<String>>{};
    for (final identity in identities) {
      for (final value in identity.values) {
        additions.putIfAbsent(_bucket(value), () => <String>{}).add(value);
      }
    }
    for (final entry in additions.entries) {
      await _mergeBucket(client, account, entry.key, entry.value);
    }
  }

  Future<void> _mergeBucket(
    DatabaseClient client,
    AccountStorageKey account,
    String bucket,
    Set<String> additions,
  ) async {
    final record = _ledger(account).record(bucket);
    final saved = await record.get(client);
    final values = saved == null
        ? <String>{}
        : _bucketMapper.fromMap(saved, bucket: bucket);
    await record.put(client, _bucketMapper.toMap(values..addAll(additions)));
  }

  Future<void> _putRecent(
    DatabaseClient client,
    AccountStorageKey account,
    WatchHistoryEntry entry,
  ) {
    final identity = VideoWatchFingerprints.fromEntry(entry);
    final value = _entryMapper.toMap(entry)
      ..['watchedAtMs'] = entry.watchedAt.toUtc().millisecondsSinceEpoch;
    return _recent(account).record(identity.target).put(client, value);
  }

  Future<void> _trimRecent(
    DatabaseClient client,
    AccountStorageKey account,
  ) async {
    final keys = await _recent(account).findKeys(
      client,
      finder: Finder(
        sortOrders: [SortOrder<int>('watchedAtMs', false)],
        offset: _recentCapacity,
      ),
    );
    if (keys.isNotEmpty) await _recent(account).records(keys).delete(client);
  }

  String _bucket(String fingerprint) =>
      fingerprint.substring(0, _bucketPrefixLength);
}
