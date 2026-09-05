import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:ghostr/platform/nostr/signed_nostr_event_json.dart';
import 'package:ndk/ndk.dart';

part 'warp_feed_relay_query.dart';

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
  var _closing = false;
  late final StreamSubscription<HttpRequest> _subscription;
  var acceptedConnections = 0;
  var requestMessages = 0;
  var videoSubscriptions = 0;
  var eventsSent = 0;
  final requestedFilters = <Map<String, Object?>>[];

  Uri get uri => Uri.parse('ws://${_server.address.address}:${_server.port}');

  void _accept(HttpRequest request) => unawaited(_upgrade(request));

  Future<void> _upgrade(HttpRequest request) async {
    if (!WebSocketTransformer.isUpgradeRequest(request)) {
      request.response.statusCode = HttpStatus.badRequest;
      return request.response.close();
    }
    final socket = await WebSocketTransformer.upgrade(request);
    if (_closing) return socket.close();
    acceptedConnections += 1;
    _sockets.add(socket);
    unawaited(_serve(socket));
  }

  Future<void> _serve(WebSocket socket) async {
    try {
      await for (final message in socket) {
        _handle(socket, message);
      }
    } finally {
      _sockets.remove(socket);
    }
  }

  Future<void> close() async {
    _closing = true;
    await _subscription.cancel();
    for (final socket in _sockets.toList()) {
      await socket.close();
    }
    await _server.close(force: true);
  }
}
