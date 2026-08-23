import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:ghostr/platform/media/ffi_playback_preparation_updates.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

void main() {
  test('maps one exact native preparation window', () async {
    final native = StreamController<FfiPlaybackPreparationPlan>();
    addTearDown(native.close);
    final updates = FfiPlaybackPreparationUpdates(watch: () => native.stream);
    final received = updates.watchPreparation().first;

    native.add(_plan());
    final plan = await received;

    expect(plan.revision, BigInt.from(7));
    expect(plan.currentDeliveryId?.value, 'current');
    expect(plan.current?.assetId.value, _currentCapability);
    expect(plan.current?.representationId.value, 'a' * 64);
    expect(plan.current?.readiness, PlaybackPreparationReadiness.preparing);
    expect(plan.next?.deliveryId.value, 'next');
    expect(plan.next?.sourceRepresentationId.value, 'd' * 64);
    expect(plan.next?.readiness, PlaybackPreparationReadiness.ready);
    expect(plan.upcoming.map((asset) => asset.deliveryId.value), [
      'next',
      'next-2',
    ]);
  });
}

FfiPlaybackPreparationPlan _plan() {
  return FfiPlaybackPreparationPlan(
    revision: BigInt.from(7),
    currentDeliveryId: 'current',
    current: _asset((
      id: 'current',
      capability: _currentCapability,
      digest: 'a',
      sourceDigest: 'a',
      readiness: FfiPlaybackPreparationReadiness.preparing,
    )),
    upcoming: [
      _asset((
        id: 'next',
        capability: _nextCapability,
        digest: 'b',
        sourceDigest: 'd',
        readiness: FfiPlaybackPreparationReadiness.ready,
      )),
      _asset((
        id: 'next-2',
        capability: _laterCapability,
        digest: 'c',
        sourceDigest: 'c',
        readiness: FfiPlaybackPreparationReadiness.structuralStartable,
      )),
    ],
    next: _asset((
      id: 'next',
      capability: _nextCapability,
      digest: 'b',
      sourceDigest: 'd',
      readiness: FfiPlaybackPreparationReadiness.ready,
    )),
  );
}

FfiPlaybackPreparationAsset _asset(_NativeAsset input) {
  return FfiPlaybackPreparationAsset(
    deliveryId: input.id,
    representationId: input.digest * 64,
    sourceRepresentationId: input.sourceDigest * 64,
    assetId: input.capability,
    playbackUrl:
        'http://127.0.0.1:17654/video.mp4?'
        'id=${input.id}&cap=${input.capability}',
    readiness: input.readiness,
  );
}

typedef _NativeAsset = ({
  String id,
  String capability,
  String digest,
  String sourceDigest,
  FfiPlaybackPreparationReadiness readiness,
});

const _currentCapability = 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA';
const _nextCapability = 'BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB';
const _laterCapability = 'CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC';
