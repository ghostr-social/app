import 'package:ndk/ndk.dart';

import 'progressive_device_origin.dart';
import 'progressive_mp4_fixture.dart';

Future<List<Nip01Event>> signedWarpMixedFeedEvents(
  ProgressiveDeviceOrigin origin,
) async {
  final signer = const Bip340EventSignerFactory().createWithNewKeyPair();
  final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
  try {
    final sources = [
      _MixedSource(origin.urlFor('current'), 'current', 'video/mp4'),
      _MixedSource(
        origin.hlsUrlFor('multivariant'),
        'hls',
        'application/vnd.apple.mpegurl',
      ),
      _MixedSource(origin.urlFor('third'), 'third', 'video/mp4'),
    ];
    return await _signSources(signer, sources, now);
  } finally {
    await signer.dispose();
  }
}

Future<List<Nip01Event>> _signSources(
  EventSigner signer,
  List<_MixedSource> sources,
  int now,
) async {
  final events = <Nip01Event>[];
  for (var index = 0; index < sources.length; index += 1) {
    events.add(await _signSource(signer, sources[index], now - index));
  }
  return events;
}

Future<Nip01Event> _signSource(
  EventSigner signer,
  _MixedSource source,
  int publishedAt,
) async {
  final event = Nip01Event(
    pubKey: signer.getPublicKey(),
    kind: 22,
    createdAt: publishedAt,
    tags: _videoTags(source, publishedAt),
    content: 'WARP signed ${source.label}',
  );
  final signed = await signer.sign(event);
  if (!await Bip340EventVerifier().verify(signed) ||
      !await RustEventVerifier().verify(signed)) {
    throw StateError('WARP mixed fixture signature is invalid.');
  }
  return signed;
}

List<List<String>> _videoTags(_MixedSource source, int publishedAt) => [
  ['title', 'WARP ${source.label}'],
  ['published_at', '$publishedAt'],
  ['alt', 'WARP signed ${source.label}'],
  [
    'imeta',
    'url ${source.url}',
    'm ${source.mime}',
    if (source.mime == 'video/mp4')
      'size ${ProgressiveMp4Fixture.bytes.length}',
    'duration 6',
    'dim 320x180',
  ],
];

final class _MixedSource {
  const _MixedSource(this.url, this.label, this.mime);

  final Uri url;
  final String label;
  final String mime;
}
