import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:ndk/ndk.dart';
import 'package:ghostr/core/nostr/signed_nostr_event_json.dart';
import 'package:ghostr/platform/nostr/signed_nostr_event_json.dart';
import 'live_video_log.dart';

// Exact, read-only public relay requests. No stored event substitutes a reply.
Future<Map<String, Nip01Event>> liveRelayRead(
  LiveVideoLog log,
  List<String> relays,
  Set<String> ids,
) async {
  final replies = await Future.wait(
    relays.map((relay) => _RelayRead(log, relay, ids).run()),
  );
  return {for (final reply in replies) ...reply};
}

final class _RelayRead {
  _RelayRead(this.log, this.relay, this.ids);
  final LiveVideoLog log;
  final String relay;
  final Set<String> ids;
  final events = <String, Nip01Event>{};
  final clock = Stopwatch()..start();
  final client = HttpClient()..connectionTimeout = const Duration(seconds: 8);
  WebSocket? socket;

  Future<Map<String, Nip01Event>> run() async {
    try {
      socket = await WebSocket.connect(
        relay,
        customClient: client,
      ).timeout(const Duration(seconds: 8));
      log.add('relay_connected', {
        'relay': relay,
        'durationMs': clock.elapsedMilliseconds,
      });
      socket!.add(
        jsonEncode([
          'REQ',
          'live-pins',
          {'ids': ids.toList()},
        ]),
      );
      await _receive();
    } on Object catch (error) {
      log.add('relay_error', {
        'relay': relay,
        'durationMs': clock.elapsedMilliseconds,
        'error': '$error',
      });
    } finally {
      if (socket?.readyState == WebSocket.open) {
        socket!.add(jsonEncode(['CLOSE', 'live-pins']));
      }
      await socket?.close();
      client.close(force: true);
    }
    log.add('relay_result', {
      'relay': relay,
      'durationMs': clock.elapsedMilliseconds,
      'eventIds': events.keys.toList(),
    });
    return events;
  }

  Future<void> _receive() async {
    final iterator = StreamIterator<Object?>(socket!);
    try {
      while (clock.elapsed < const Duration(seconds: 15)) {
        final remaining = const Duration(seconds: 15) - clock.elapsed;
        if (!await iterator.moveNext().timeout(remaining)) break;
        if (await _message(iterator.current)) break;
      }
    } finally {
      await iterator.cancel();
    }
  }

  Future<bool> _message(Object? raw) async {
    if (raw is! String || raw.length > 1024 * 1024) return false;
    final message = jsonDecode(raw) as List<Object?>;
    if (message.length < 2) return false;
    if (message[1] != 'live-pins') return false;
    if (message[0] == 'EOSE' || message[0] == 'CLOSED') return true;
    if (message[0] != 'EVENT' || message.length != 3) return false;
    final event = decodeSignedNostrEvent(
      SignedNostrEventJson.parse(jsonEncode(message[2])),
    );
    if (!ids.contains(event.id)) return false;
    if (!await RustEventVerifier().verify(event)) {
      throw StateError('Invalid signature from $relay');
    }
    events[event.id] = event;
    return false;
  }
}
