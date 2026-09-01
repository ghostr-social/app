import 'package:ndk/ndk.dart';

import 'progressive_device_origin.dart';
import 'progressive_mp4_fixture.dart';
import 'warp_no_video_rendition_fixture.dart';

Future<List<Nip01Event>> signedInvalidTrackFallbackEvents(
  ProgressiveDeviceOrigin origin,
  WarpNoVideoRenditionFixture fixture,
) async {
  final signer = const Bip340EventSignerFactory().createWithNewKeyPair();
  try {
    final event = Nip01Event(
      pubKey: signer.getPublicKey(),
      kind: 22,
      createdAt: DateTime.now().millisecondsSinceEpoch ~/ 1000,
      tags: _renditionTags(origin, fixture),
      content: 'WARP invalid-track rendition fallback',
    );
    final signed = await signer.sign(event);
    await _requireValid(signed);
    return [signed];
  } finally {
    await signer.dispose();
  }
}

List<List<String>> _renditionTags(
  ProgressiveDeviceOrigin origin,
  WarpNoVideoRenditionFixture fixture,
) => [
  ['title', 'WARP rendition recovery'],
  ['alt', 'WARP rendition recovery'],
  _invalidRenditionTag(origin, fixture),
  _validRenditionTag(origin),
];

List<String> _invalidRenditionTag(
  ProgressiveDeviceOrigin origin,
  WarpNoVideoRenditionFixture fixture,
) => [
  'imeta',
  'url ${fixture.urlFor(origin)}',
  'm video/mp4',
  'bitrate 6000000',
  'size 810',
  'duration 1',
];

List<String> _validRenditionTag(ProgressiveDeviceOrigin origin) => [
  'imeta',
  'url ${origin.urlFor('valid-rendition')}',
  'm video/mp4',
  'bitrate 1000000',
  'size ${ProgressiveMp4Fixture.bytes.length}',
  'duration 6',
  'dim 320x180',
];

Future<void> _requireValid(Nip01Event event) async {
  if (!await Bip340EventVerifier().verify(event)) {
    throw StateError('Invalid WARP rendition fixture signature.');
  }
  if (!await RustEventVerifier().verify(event)) {
    throw StateError('Native verifier rejected WARP rendition fixture.');
  }
}
