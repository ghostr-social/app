import 'package:ndk/ndk.dart';

import 'progressive_device_origin.dart';
import 'progressive_mp4_fixture.dart';

Future<List<Nip01Event>> signedUnsupportedHlsRescueEvents(
  ProgressiveDeviceOrigin origin,
) async {
  final signer = const Bip340EventSignerFactory().createWithNewKeyPair();
  final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
  try {
    return [
      await _sign(signer, now, _unsupportedTags(origin)),
      await _sign(signer, now - 1, _alternateTags(origin)),
    ];
  } finally {
    await signer.dispose();
  }
}

Future<Nip01Event> _sign(
  EventSigner signer,
  int createdAt,
  List<List<String>> tags,
) async {
  final event = Nip01Event(
    pubKey: signer.getPublicKey(),
    kind: 22,
    createdAt: createdAt,
    tags: tags,
    content: tags[1][1],
  );
  final signed = await signer.sign(event);
  if (!await Bip340EventVerifier().verify(signed) ||
      !await RustEventVerifier().verify(signed)) {
    throw StateError('Unsupported-HLS fixture signature is invalid.');
  }
  return signed;
}

List<List<String>> _unsupportedTags(ProgressiveDeviceOrigin origin) => [
  ['title', 'WARP unsupported encrypted HLS'],
  ['alt', 'WARP unsupported encrypted HLS'],
  [
    'imeta',
    'url ${origin.encryptedHlsUrl}',
    'm application/vnd.apple.mpegurl',
    'duration 6',
    'dim 320x180',
  ],
];

List<List<String>> _alternateTags(ProgressiveDeviceOrigin origin) => [
  ['title', 'WARP decoded rescue alternate'],
  ['alt', 'WARP decoded rescue alternate'],
  [
    'imeta',
    'url ${origin.urlFor('unsupported-hls-rescue')}',
    'm video/mp4',
    'size ${ProgressiveMp4Fixture.bytes.length}',
    'duration 6',
    'dim 320x180',
  ],
];
