import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';

typedef WarpFeedDeliveryClock = Duration Function();
typedef WarpFeedDeliverySequence = int Function();

final class WarpFeedDeliveryObservation {
  const WarpFeedDeliveryObservation(this.elapsed, this.sequence, this.snapshot);

  final Duration elapsed;
  final int sequence;
  final VideoDeliverySnapshot snapshot;
}

final class WarpFeedDeliveryProbe implements VideoDeliveryUpdates {
  WarpFeedDeliveryProbe(this._delegate, this._clock, this._sequence);

  final VideoDeliveryUpdates _delegate;
  final WarpFeedDeliveryClock _clock;
  final WarpFeedDeliverySequence _sequence;
  final _observations = <WarpFeedDeliveryObservation>[];

  List<WarpFeedDeliveryObservation> get observations =>
      List.unmodifiable(_observations);

  String get evidence => _observations.reversed
      .take(24)
      .map((item) {
        final snapshot = item.snapshot;
        return '${snapshot.deliveryId.value}:${snapshot.phase.name}:'
            '${snapshot.bytesPresent}:${snapshot.eta?.inMilliseconds ?? 'na'}:'
            '${item.sequence}';
      })
      .join('|');

  @override
  Stream<VideoDeliverySnapshot> watchDelivery() {
    return _delegate.watchDelivery().map((snapshot) {
      _observations.add(
        WarpFeedDeliveryObservation(_clock(), _sequence(), snapshot),
      );
      return snapshot;
    });
  }
}
