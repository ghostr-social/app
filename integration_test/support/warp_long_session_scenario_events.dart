part of 'warp_long_session_scenario.dart';

const _longSessionPostCount = 24;

Future<List<Nip01Event>> _longSessionEvents(
  ProgressiveDeviceOrigin origin,
) async {
  final signer = const Bip340EventSignerFactory().createWithNewKeyPair();
  final published = DateTime.now().millisecondsSinceEpoch ~/ 1000;
  try {
    final events = <Nip01Event>[];
    for (var index = 0; index < _longSessionPostCount; index += 1) {
      events.add(await _longSessionEvent(signer, origin, index, published));
    }
    return events;
  } finally {
    await signer.dispose();
  }
}

Future<Nip01Event> _longSessionEvent(
  EventSigner signer,
  ProgressiveDeviceOrigin origin,
  int index,
  int published,
) async {
  final label = _longSessionLabel(index);
  final event = Nip01Event(
    pubKey: signer.getPublicKey(),
    kind: 22,
    createdAt: published - index,
    tags: _longSessionTags(origin.urlFor(label), published - index),
    content: 'WARP long session $label',
  );
  final signed = await signer.sign(event);
  await _requireValidLongSessionEvent(signed);
  return signed;
}

Future<void> _requireValidLongSessionEvent(Nip01Event event) async {
  if (!await Bip340EventVerifier().verify(event)) {
    throw StateError('Long-session fixture signature is invalid.');
  }
  if (!await RustEventVerifier().verify(event)) {
    throw StateError('Long-session fixture fails native verification.');
  }
}

List<List<String>> _longSessionTags(Uri url, int published) => [
  ['title', 'WARP long session'],
  ['published_at', '$published'],
  ['alt', 'WARP long-session boundedness fixture'],
  [
    'imeta',
    'url $url',
    'm video/mp4',
    'size ${ProgressiveMp4Fixture.bytes.length}',
    'duration 6',
    'dim 320x180',
  ],
];

String _longSessionLabel(int index) =>
    'long-${index.toString().padLeft(2, '0')}';
