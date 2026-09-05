part of 'warp_feed_relay.dart';

extension _WarpFeedRelayQuery on WarpFeedRelay {
  void _handle(WebSocket socket, Object? raw) {
    if (_closing) return;
    if (socket.readyState != WebSocket.open || raw is! String) return;
    final message = jsonDecode(raw);
    if (message is! List || message.length < 2 || message.first != 'REQ') {
      return;
    }
    requestMessages += 1;
    final subscription = message[1];
    if (subscription is! String) return;
    final filters = _filters(message);
    requestedFilters.addAll(filters);
    _sendMatches(socket, subscription, filters);
    socket.add(jsonEncode(['EOSE', subscription]));
  }

  List<Map<String, Object?>> _filters(List<dynamic> message) {
    return message
        .skip(2)
        .whereType<Map>()
        .map((filter) => filter.cast<String, Object?>())
        .toList();
  }

  void _sendMatches(
    WebSocket socket,
    String subscription,
    List<Map<String, Object?>> filters,
  ) {
    if (!_requestsKind(filters, 22)) return;
    videoSubscriptions += 1;
    final matches = _events.where((event) => _matchesAny(event, filters));
    for (final event in matches) {
      socket.add(jsonEncode(['EVENT', subscription, _payload(event)]));
      eventsSent += 1;
    }
  }

  bool _requestsKind(Iterable<Map<String, Object?>> filters, int kind) {
    return filters.any((filter) {
      final kinds = filter['kinds'];
      return kinds is List && kinds.contains(kind);
    });
  }

  bool _matchesAny(Nip01Event event, Iterable<Map<String, Object?>> filters) {
    return filters.any((filter) => _matches(event, filter));
  }

  bool _matches(Nip01Event event, Map<String, Object?> filter) {
    final kinds = filter['kinds'];
    if (kinds is List && !kinds.contains(event.kind)) return false;
    final authors = filter['authors'];
    if (authors is List && !authors.contains(event.pubKey)) return false;
    final until = filter['until'];
    if (until is num && event.createdAt > until.toInt()) return false;
    final since = filter['since'];
    return since is! num || event.createdAt >= since.toInt();
  }

  Object? _payload(Nip01Event event) {
    return jsonDecode(encodeSignedNostrEvent(event).value);
  }
}
