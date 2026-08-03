import 'dart:developer';

import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/time/clock.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ndk/ndk.dart';

/// NIP-65 outbox routing: content lives on the relays its authors write to,
/// so queries go there instead of only the bootstrap set.
class NdkNostrOutboxDirectory {
  NdkNostrOutboxDirectory(
    this._ndk, {
    List<RelayUrl> bootstrapRelays = const [],
    Clock clock = systemClock,
    int maxOutboxRelays = 12,
  })  : _bootstrapUrls = List<String>.unmodifiable(
          bootstrapRelays.map((relay) => relay.value),
        ),
        _clock = clock,
        _maxOutboxRelays = maxOutboxRelays;

  static const _timeToLive = Duration(minutes: 30);

  final Ndk _ndk;
  final List<String> _bootstrapUrls;
  final Clock _clock;
  final int _maxOutboxRelays;
  List<String>? _cachedDiscovery;
  DateTime? _cachedAt;

  /// Relays where the signed-in account's follows publish, ranked by how
  /// many follows write there, merged with the bootstrap relays.
  Future<List<String>> discoveryRelayUrls() async {
    final cached = _cachedDiscovery;
    final cachedAt = _cachedAt;
    if (cached != null &&
        cachedAt != null &&
        _clock().difference(cachedAt) < _timeToLive) {
      return cached;
    }
    final relays = await _guarded(
      () async => _merged(await _rankedWriteRelays(await _followedPubkeys())),
    );
    _cachedDiscovery = relays;
    _cachedAt = _clock();
    return relays;
  }

  /// Write relays declared by [authors] themselves — where their videos are
  /// guaranteed to be found — merged with the bootstrap relays.
  Future<List<String>> authorWriteRelayUrls(
    Set<NostrPublicKeyHex> authors,
  ) async {
    return _guarded(() async {
      final pubkeys = authors.map((author) => author.value).toList();
      return _merged(await _rankedWriteRelays(pubkeys));
    });
  }

  Future<List<String>> _followedPubkeys() async {
    final publicKey = _ndk.accounts.getPublicKey();
    if (publicKey == null) return const <String>[];
    final contacts = await _ndk.follows.getContactList(publicKey);
    return contacts?.contacts ?? const <String>[];
  }

  Future<List<String>> _rankedWriteRelays(List<String> pubkeys) async {
    if (pubkeys.isEmpty) return const <String>[];
    await _ndk.userRelayLists.loadMissingRelayListsFromNip65OrNip02(pubkeys);
    final counts = <String, int>{};
    for (final publicKey in pubkeys) {
      for (final url in await _writeUrls(publicKey)) {
        counts[url] = (counts[url] ?? 0) + 1;
      }
    }
    final ranked = counts.keys.toList()
      ..sort((left, right) {
        final byCount = counts[right]!.compareTo(counts[left]!);
        return byCount == 0 ? left.compareTo(right) : byCount;
      });
    return ranked.take(_maxOutboxRelays).toList();
  }

  Future<Set<String>> _writeUrls(String publicKey) async {
    final list = await _ndk.userRelayLists.getSingleUserRelayList(publicKey);
    if (list == null) return const <String>{};
    return list.writeUrls
        .map(RelayUrl.tryParse)
        .whereType<RelayUrl>()
        .map((relay) => relay.value)
        .toSet();
  }

  List<String> _merged(Iterable<String> outbox) {
    return List<String>.unmodifiable(<String>{..._bootstrapUrls, ...outbox});
  }

  // Outbox data is an optimization; without it queries still reach the
  // bootstrap relays.
  Future<List<String>> _guarded(Future<List<String>> Function() load) async {
    try {
      return await load();
    } on Object catch (error, stackTrace) {
      log(
        'Outbox relays unavailable; falling back to bootstrap relays.',
        name: 'ghostr.nostr.outbox',
        error: error,
        stackTrace: stackTrace,
      );
      return _bootstrapUrls;
    }
  }
}
