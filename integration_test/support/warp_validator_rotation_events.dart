import 'package:ndk/ndk.dart';

import 'warp_validator_rotation_fixture.dart';

Future<List<Nip01Event>> signedWarpValidatorRotationEvents(
  WarpValidatorRotationFixture fixture,
) async {
  final signer = const Bip340EventSignerFactory().createWithNewKeyPair();
  final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
  try {
    final events = <Nip01Event>[];
    for (var index = 0; index < _rotationLabels.length; index += 1) {
      final event = _rotationEvent((
        fixture: fixture,
        signer: signer,
        now: now,
        index: index,
      ));
      final signed = await signer.sign(event);
      if (!await Bip340EventVerifier().verify(signed) ||
          !await RustEventVerifier().verify(signed)) {
        throw StateError('Validator-rotation event signature is invalid.');
      }
      events.add(signed);
    }
    return events;
  } finally {
    await signer.dispose();
  }
}

Nip01Event _rotationEvent(
  ({
    WarpValidatorRotationFixture fixture,
    EventSigner signer,
    int now,
    int index,
  })
  input,
) {
  final label = _rotationLabels[input.index];
  final url = input.index == 0
      ? input.fixture.mediaUrl
      : input.fixture.stableUrl;
  return Nip01Event(
    pubKey: input.signer.getPublicKey(),
    kind: 22,
    createdAt: input.now - input.index,
    tags: _rotationTags(
      label,
      url,
      input.fixture.firstBytes.length,
      input.now - input.index,
    ),
    content: 'WARP signed $label',
  );
}

List<List<String>> _rotationTags(
  String label,
  Uri url,
  int length,
  int publishedAt,
) => [
  ['title', 'WARP $label'],
  ['published_at', '$publishedAt'],
  ['alt', 'WARP signed $label'],
  [
    'imeta',
    'url $url',
    'm video/mp4',
    'size $length',
    'duration 6',
    'dim 320x180',
  ],
];

const _rotationLabels = ['rotating', 'stable'];
