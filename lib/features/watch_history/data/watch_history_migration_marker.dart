import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:sembast/sembast.dart';

enum WatchHistoryMigrationState { migrated, cleared }

final class WatchHistoryMigrationMarker {
  const WatchHistoryMigrationMarker.migrated(this.ordinaryPublishedThrough)
    : state = WatchHistoryMigrationState.migrated;

  const WatchHistoryMigrationMarker.cleared()
    : state = WatchHistoryMigrationState.cleared,
      ordinaryPublishedThrough = null;

  factory WatchHistoryMigrationMarker.fromMap(Map<String, Object?> value) {
    _requireSchema(value);
    final state = _stateFrom(value['state']);
    final cutoff = _cutoffFrom(value['ordinaryPublishedThroughMs']);
    return _fromState(state, cutoff);
  }

  static const _invalidMarker = FormatException(
    'Watch history migration marker is invalid.',
  );

  final WatchHistoryMigrationState state;
  final DateTime? ordinaryPublishedThrough;

  static void _requireSchema(Map<String, Object?> value) {
    if (value['schema'] != 1) throw _invalidMarker;
  }

  static WatchHistoryMigrationState _stateFrom(Object? raw) {
    if (raw == 'migrated') return WatchHistoryMigrationState.migrated;
    if (raw == 'cleared') return WatchHistoryMigrationState.cleared;
    throw _invalidMarker;
  }

  static DateTime? _cutoffFrom(Object? raw) {
    if (raw == null) return null;
    if (raw is! int) throw _invalidMarker;
    if (raw <= 0) throw _invalidMarker;
    return DateTime.fromMillisecondsSinceEpoch(raw, isUtc: true);
  }

  static WatchHistoryMigrationMarker _fromState(
    WatchHistoryMigrationState state,
    DateTime? cutoff,
  ) {
    if (state == WatchHistoryMigrationState.cleared) {
      if (cutoff != null) throw _invalidMarker;
      return const WatchHistoryMigrationMarker.cleared();
    }
    return WatchHistoryMigrationMarker.migrated(cutoff);
  }

  Map<String, Object?> toMap() => <String, Object?>{
    'schema': 1,
    'state': state.name,
    if (ordinaryPublishedThrough case final cutoff?)
      'ordinaryPublishedThroughMs': cutoff.toUtc().millisecondsSinceEpoch,
  };
}

final class WatchHistoryMigrationStore {
  const WatchHistoryMigrationStore();

  static final _store = stringMapStoreFactory.store('watch_meta_v1');

  Future<WatchHistoryMigrationMarker?> load(
    DatabaseClient client,
    AccountStorageKey account,
  ) async {
    final value = await _store.record(account.account).get(client);
    return value == null ? null : WatchHistoryMigrationMarker.fromMap(value);
  }

  Future<WatchHistoryMigrationMarker> require(
    DatabaseClient client,
    AccountStorageKey account,
  ) async {
    final marker = await load(client, account);
    if (marker == null) {
      throw const FormatException('Watch history is unready.');
    }
    return marker;
  }

  Future<void> put(
    DatabaseClient client,
    AccountStorageKey account,
    WatchHistoryMigrationMarker marker,
  ) {
    return _store.record(account.account).put(client, marker.toMap());
  }
}
