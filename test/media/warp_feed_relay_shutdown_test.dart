import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ndk/ndk.dart';

import '../../integration_test/support/warp_feed_relay.dart';

void main() {
  test('relay shutdown during a response does not write after close', () async {
    final signer = const Bip340EventSignerFactory().createWithNewKeyPair();
    addTearDown(signer.dispose);
    final event = await signer.sign(
      Nip01Event(
        pubKey: signer.getPublicKey(),
        kind: 22,
        content: 'video',
        tags: const [],
      ),
    );
    final relay = await WarpFeedRelay.start([event]);
    final closing = Completer<void>();
    final socket = await WebSocket.connect(relay.uri.toString());
    addTearDown(socket.close);
    socket.add(
      jsonEncode([
        'REQ',
        'videos',
        {
          'kinds': [22],
        },
      ]),
    );
    final responses = <Object?>[];
    await for (final raw in socket) {
      final response = jsonDecode(raw as String) as List;
      responses.add(response);
      if (response.first == 'EVENT') {
        socket.add(
          jsonEncode([
            'REQ',
            'queued',
            {
              'kinds': [0],
            },
          ]),
        );
        closing.complete(relay.close());
      }
    }
    await closing.future;
    expect(relay.videoSubscriptions, 1);
    expect(responses, hasLength(2));
    expect(responses.last, ['EOSE', 'videos']);
  });
}
