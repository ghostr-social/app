import 'package:ndk/ndk.dart';

import 'progressive_device_origin.dart';
import 'progressive_mp4_fixture.dart';

Future<List<Nip01Event>> signedWarpFeedEvents(
  ProgressiveDeviceOrigin origin,
) async {
  final signer = const Bip340EventSignerFactory().createWithNewKeyPair();
  final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
  try {
    final events = <Nip01Event>[];
    for (var index = 0; index < 3; index += 1) {
      final label = _labels[index];
      final event = Nip01Event(
        pubKey: signer.getPublicKey(),
        kind: 22,
        createdAt: now - index,
        tags: _videoTags(origin.urlFor(label), label, now - index),
        content: 'WARP signed $label',
      );
      final signed = await signer.sign(event);
      if (!await Bip340EventVerifier().verify(signed)) {
        throw StateError('WARP fixture event signature is invalid.');
      }
      events.add(signed);
    }
    return events;
  } finally {
    await signer.dispose();
  }
}

const _labels = ['current', 'next', 'third'];

List<List<String>> _videoTags(Uri url, String label, int publishedAt) => [
  ['title', 'WARP $label'],
  ['published_at', '$publishedAt'],
  ['alt', 'WARP signed $label'],
  [
    'imeta',
    'url $url',
    'm video/mp4',
    'size ${ProgressiveMp4Fixture.bytes.length}',
    'duration 6',
    'dim 64x64',
  ],
];
