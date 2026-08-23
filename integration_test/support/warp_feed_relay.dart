import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:ghostr/platform/nostr/signed_nostr_event_json.dart';
import 'package:ndk/ndk.dart';

final class WarpFeedRelay {
  WarpFeedRelay._(this._server, this._events);

  static Future<WarpFeedRelay> start(List<Nip01Event> events) async {
    final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
    final relay = WarpFeedRelay._(server, events);
    relay._subscription = server.listen(relay._accept);
    return relay;
  }

  final HttpServer _server;
  final List<Nip01Event> _events;
  final _sockets = <WebSocket>{};
  late final StreamSubscription<HttpRequest> _subscription;
  var acceptedConnections = 0;
  var requestMessages = 0;
  var videoSubscriptions = 0;

  Uri get uri => Uri.parse('ws://${_server.address.address}:${_server.port}');

  void _accept(HttpRequest request) => unawaited(_upgrade(request));

  Future<void> _upgrade(HttpRequest request) async {
    if (!WebSocketTransformer.isUpgradeRequest(request)) {
      request.response.statusCode = HttpStatus.badRequest;
      return request.response.close();
    }
    final socket = await WebSocketTransformer.upgrade(request);
    acceptedConnections += 1;
    _sockets.add(socket);
    unawaited(_serve(socket));
  }

  Future<void> _serve(WebSocket socket) async {
    try {
      await for (final message in socket) {
        await _handle(socket, message);
      }
    } finally {
      _sockets.remove(socket);
    }
  }

  Future<void> _handle(WebSocket socket, Object? raw) async {
    if (raw is! String) return;
    final message = jsonDecode(raw);
    if (message is! List || message.length < 2 || message.first != 'REQ') {
      return;
    }
    requestMessages += 1;
    final subscription = message[1];
    if (subscription is! String) return;
    final filters = message
        .skip(2)
        .whereType<Map>()
        .map((filter) => filter.cast<String, Object?>());
    if (_requestsKind(filters, 22)) {
      videoSubscriptions += 1;
      for (final event in _events) {
        socket.add(jsonEncode(['EVENT', subscription, _payload(event)]));
      }
      await Future<void>.delayed(Duration.zero);
    }
    socket.add(jsonEncode(['EOSE', subscription]));
  }

  bool _requestsKind(Iterable<Map<String, Object?>> filters, int kind) {
    return filters.any((filter) {
      final kinds = filter['kinds'];
      return kinds is List && kinds.contains(kind);
    });
  }

  Object? _payload(Nip01Event event) {
    return jsonDecode(encodeSignedNostrEvent(event).value);
  }

  Future<void> close() async {
    for (final socket in _sockets.toList()) {
      await socket.close();
    }
    await _subscription.cancel();
    await _server.close(force: true);
  }
}
