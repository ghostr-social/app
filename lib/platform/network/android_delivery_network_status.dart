import 'dart:async';

import 'package:flutter/services.dart';
import 'package:ghostr/core/network/delivery_network_status.dart';
import 'package:ghostr/core/network/delivery_network_status_port.dart';

final class AndroidDeliveryNetworkStatus implements DeliveryNetworkStatusPort {
  AndroidDeliveryNetworkStatus({MethodChannel? channel})
    : _channel = channel ?? const MethodChannel(channelName) {
    _channel.setMethodCallHandler(_handleNativeCall);
  }

  static const channelName = 'social.ghostr/network/v1';

  final MethodChannel _channel;
  final _changes = StreamController<DeliveryNetworkStatus>.broadcast();
  DeliveryNetworkStatus? _latest;
  bool _closed = false;

  @override
  Stream<DeliveryNetworkStatus> get changes => _changes.stream;

  @override
  Future<DeliveryNetworkStatus> read() async {
    final raw = await _channel.invokeMethod<Object?>('readNetworkStatus');
    _record(_status(raw), publish: false);
    return _latest ?? DeliveryNetworkStatus.unavailable;
  }

  @override
  Future<void> close() async {
    if (_closed) return;
    _closed = true;
    _channel.setMethodCallHandler(null);
    await _changes.close();
  }

  Future<Object?> _handleNativeCall(MethodCall call) async {
    if (call.method != 'networkStatusChanged') throw MissingPluginException();
    _record(_status(call.arguments), publish: true);
    return null;
  }

  void _record(DeliveryNetworkStatus status, {required bool publish}) {
    final latest = _latest;
    if (latest != null && !status.isFresherThan(latest)) return;
    _latest = status;
    if (publish && !_closed) _changes.add(status);
  }
}

DeliveryNetworkStatus _status(Object? raw) {
  if (raw is! Map<Object?, Object?>) {
    throw const FormatException('Android network status must be a map.');
  }
  final value = raw['class'];
  final generation = raw['generation'];
  if (value is! String || generation is! int || generation < 0) {
    throw const FormatException('Android network status is invalid.');
  }
  return DeliveryNetworkStatus(_networkClass(value), generation: generation);
}

DeliveryNetworkClass _networkClass(String value) => switch (value) {
  'unavailable' => DeliveryNetworkClass.unavailable,
  'wifi' => DeliveryNetworkClass.wifi,
  'cellular' => DeliveryNetworkClass.cellular,
  'wired' => DeliveryNetworkClass.wired,
  'constrained' => DeliveryNetworkClass.constrained,
  _ => throw FormatException('Unknown Android network class: $value'),
};
