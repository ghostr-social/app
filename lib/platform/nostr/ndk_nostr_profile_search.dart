import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/video_catalog/data/nostr_profile_search_port.dart';
import 'package:ndk/ndk.dart';

class NdkNostrProfileSearch implements NostrProfileSearchPort {
  NdkNostrProfileSearch(
    this._ndk, {
    List<RelayUrl> searchRelays = const [],
    int limit = 30,
  })  : _searchRelayUrls = List<String>.unmodifiable(
          searchRelays.map((relay) => relay.value),
        ),
        _limit = limit;

  static const _timeout = Duration(seconds: 5);
  static final _hexKeyPattern = RegExp(r'^[0-9a-f]{64}$');

  final Ndk _ndk;
  final List<String> _searchRelayUrls;
  final int _limit;

  @override
  Future<List<Metadata>> searchProfiles(String query) async {
    try {
      final events = await _response(query.trim()).future;
      return _newestUniqueMetadata(events);
    } on Object catch (error, stackTrace) {
      throw _failure(error, stackTrace);
    }
  }

  NdkResponse _response(String query) {
    final author = _authorHex(query);
    final filter = author == null
        ? (Filter(kinds: const [0], limit: _limit)..search = query)
        : Filter(kinds: const [0], authors: [author], limit: 1);
    // Direct lookups work on any relay; text search needs NIP-50 indexes.
    if (author != null || _searchRelayUrls.isEmpty) {
      return _ndk.requests.query(
          name: 'ghostr-profile-search', timeout: _timeout, filter: filter);
    }
    return _ndk.requests.query(
      name: 'ghostr-profile-search',
      timeout: _timeout,
      filter: filter,
      explicitRelays: _searchRelayUrls,
    );
  }

  String? _authorHex(String query) {
    if (_hexKeyPattern.hasMatch(query)) return query;
    if (!query.startsWith('npub1')) return null;
    try {
      final hex = Nip19.decode(query);
      return _hexKeyPattern.hasMatch(hex) ? hex : null;
    } on Object {
      return null;
    }
  }

  List<Metadata> _newestUniqueMetadata(List<Nip01Event> events) {
    final newest = <String, Nip01Event>{};
    for (final event in events.where((event) => event.kind == 0)) {
      final current = newest[event.pubKey];
      if (current == null || event.createdAt > current.createdAt) {
        newest[event.pubKey] = event;
      }
    }
    return newest.values.map(Metadata.fromEvent).toList();
  }

  AppFailure _failure(Object error, StackTrace stackTrace) {
    return translatedBoundaryFailure(
      source: 'ghostr.nostr.profile-search',
      message: 'Could not search Nostr profiles.',
      error: error,
      stackTrace: stackTrace,
    );
  }
}
