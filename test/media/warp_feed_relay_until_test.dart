import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ndk/ndk.dart';

import '../../integration_test/support/warp_feed_relay.dart';

void main() {
  test('the WARP relay does not replay videos newer than until', () async {
    final signer = const Bip340EventSignerFactory().createWithNewKeyPair();
    addTearDown(signer.dispose);
    final events = await Future.wait(
      [10, 9].map((createdAt) {
        return signer.sign(
          Nip01Event(
            pubKey: signer.getPublicKey(),
            kind: 22,
            createdAt: createdAt,
            tags: const [],
            content: 'video',
          ),
        );
      }),
    );
    final relay = await WarpFeedRelay.start(events);
    addTearDown(relay.close);
    final socket = await WebSocket.connect(relay.uri.toString());
    addTearDown(socket.close);

    socket.add(
      jsonEncode([
        'REQ',
        'older',
        {
          'kinds': [22],
          'until': 8,
        },
      ]),
    );
    final raw = await socket.first as String;

    expect(jsonDecode(raw), ['EOSE', 'older']);
    expect(relay.eventsSent, 0);
  });
}
