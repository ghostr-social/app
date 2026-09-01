import 'package:ndk/ndk.dart';

import 'progressive_device_origin.dart';
import 'progressive_mp4_fixture.dart';
import 'warp_feed_event_config.dart';

Future<List<Nip01Event>> signedWarpFeedEvents(
  ProgressiveDeviceOrigin origin, {
  SignedWarpFeedConfig config = const SignedWarpFeedConfig(),
}) async {
  final count = config.eventCount;
  RangeError.checkValueInInterval(count, 1, _labels.length, 'eventCount');
  final signer = const Bip340EventSignerFactory().createWithNewKeyPair();
  final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
  try {
    final events = <Nip01Event>[];
    for (var index = 0; index < count; index += 1) {
      final label = _labels[index];
      final sources = config.sourcesFor(label);
      final event = Nip01Event(
        pubKey: signer.getPublicKey(),
        kind: 22,
        createdAt: now - index,
        tags: _videoTags(origin, sources, now - index),
        content: 'WARP signed $label',
      );
      final signed = await signer.sign(event);
      if (!await Bip340EventVerifier().verify(signed)) {
        throw StateError('WARP fixture event signature is invalid.');
      }
      if (!await RustEventVerifier().verify(signed)) {
        throw StateError('WARP fixture event fails native verification.');
      }
      events.add(signed);
    }
    return events;
  } finally {
    await signer.dispose();
  }
}

const _labels = [
  'current',
  'next',
  'third',
  'fourth',
  'fifth',
  'sixth',
  'seventh',
  'eighth',
  'ninth',
  'tenth',
];

List<List<String>> _videoTags(
  ProgressiveDeviceOrigin origin,
  WarpFeedEventSources sources,
  int publishedAt,
) => [
  ['title', 'WARP ${sources.primaryLabel}'],
  ['published_at', '$publishedAt'],
  ['alt', 'WARP signed ${sources.primaryLabel}'],
  [
    'imeta',
    'url ${origin.urlFor(sources.primaryLabel)}',
    if (sources.fallbackLabel case final fallback?)
      'fallback ${origin.urlFor(fallback)}',
    'm video/mp4',
    'size ${ProgressiveMp4Fixture.bytes.length}',
    'duration 6',
    'dim 320x180',
  ],
];
