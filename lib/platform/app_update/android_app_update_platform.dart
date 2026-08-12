import 'dart:async';

import 'package:flutter/services.dart';
import 'package:ghostr/platform/app_update/android_app_update_models.dart';

export 'android_app_update_models.dart';

final class AndroidAppUpdatePlatform {
  AndroidAppUpdatePlatform({MethodChannel? channel})
    : _channel = channel ?? const MethodChannel(channelName) {
    _channel.setMethodCallHandler(_handleNativeCall);
  }

  static const channelName = 'social.ghostr/app_update/v1';

  final MethodChannel _channel;
  final StreamController<AndroidInstallStatus> _statuses =
      StreamController<AndroidInstallStatus>.broadcast();

  Stream<AndroidInstallStatus> get statuses => _statuses.stream;

  Future<AndroidInstalledApp> getInstalledApp() async {
    final raw = await _channel.invokeMethod<Object?>('getInstalledApp');
    final payload = _map(raw, 'installed app');
    return AndroidInstalledApp(
      packageName: _string(payload['packageName'], 'packageName'),
      versionCode: _integer(payload['versionCode'], 'versionCode'),
      versionName: _string(payload['versionName'], 'versionName'),
      sdkInt: _integer(payload['sdkInt'], 'sdkInt'),
      supportedAbis: _strings(payload['supportedAbis'], 'supportedAbis'),
    );
  }

  Future<AndroidNetworkAccess> getNetworkAccess() async {
    final value = await _channel.invokeMethod<String>('getNetworkAccess');
    return switch (value) {
      'none' => AndroidNetworkAccess.none,
      'wifi' => AndroidNetworkAccess.wifi,
      'other' => AndroidNetworkAccess.other,
      _ => throw FormatException('Unknown Android network access: $value'),
    };
  }

  Future<bool> canRequestInstalls() async {
    final value = await _channel.invokeMethod<bool>('canRequestInstalls');
    return value ?? false;
  }

  Future<void> openInstallPermissionSettings() {
    return _channel.invokeMethod<void>('openInstallPermissionSettings');
  }

  Future<int> install(AndroidInstallRequest request) async {
    return _invokeInstall('install', _installArguments(request));
  }

  Future<int> replaceInstall(int sessionId, AndroidInstallRequest request) {
    return _invokeInstall('replaceInstall', {
      'sessionId': sessionId,
      ..._installArguments(request),
    });
  }

  Future<int> _invokeInstall(
    String method,
    Map<String, Object> arguments,
  ) async {
    final session = await _channel.invokeMethod<int>(method, arguments);
    if (session == null) {
      throw const FormatException('Android install did not return a session.');
    }
    return session;
  }

  Future<AndroidInstallStatus?> readInstallStatus(int sessionId) async {
    final raw = await _channel.invokeMethod<Object?>('readInstallStatus', {
      'sessionId': sessionId,
    });
    return raw == null ? null : _status(raw);
  }

  void dispose() {
    _channel.setMethodCallHandler(null);
    unawaited(_statuses.close());
  }

  Future<Object?> _handleNativeCall(MethodCall call) async {
    if (call.method != 'installStatus') throw MissingPluginException();
    _statuses.add(_status(call.arguments));
    return null;
  }
}

AndroidInstallStatus _status(Object? raw) {
  final payload = _map(raw, 'install status');
  return AndroidInstallStatus(
    sessionId: _integer(payload['sessionId'], 'sessionId'),
    state: _state(_string(payload['status'], 'status')),
    message: _nullableString(payload['message'], 'message'),
  );
}

AndroidInstallState _state(String value) => switch (value) {
  'pendingUserAction' => AndroidInstallState.pendingUserAction,
  'succeeded' => AndroidInstallState.succeeded,
  'failed' => AndroidInstallState.failed,
  _ => throw FormatException('Unknown Android install status: $value'),
};

Map<Object?, Object?> _map(Object? value, String field) {
  if (value is! Map<Object?, Object?>) {
    throw FormatException('Android $field must be a map.');
  }
  return value;
}

String _string(Object? value, String field) {
  if (value is! String) throw FormatException('$field must be text.');
  return value;
}

String? _nullableString(Object? value, String field) {
  if (value == null) return null;
  return _string(value, field);
}

int _integer(Object? value, String field) {
  if (value is! int) throw FormatException('$field must be an integer.');
  return value;
}

List<String> _strings(Object? value, String field) {
  if (value is! List<Object?>) {
    throw FormatException('$field must be a list.');
  }
  return List.unmodifiable(value.map((entry) => _string(entry, field)));
}

Map<String, Object> _installArguments(AndroidInstallRequest request) => {
  'path': request.path,
  'expectedVersionCode': request.expectedVersionCode,
  'automatic': request.mode == AndroidInstallMode.automatic,
};
