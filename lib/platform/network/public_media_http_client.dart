import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/network/public_media_address_resolver.dart';
import 'package:http/http.dart' as http;
import 'package:http/io_client.dart';

part 'public_media_connection_factory.dart';
part 'public_media_connection_race.dart';
part 'public_media_secure_connection.dart';
part 'raw_secure_socket_adapter.dart';
part 'raw_secure_socket_consumer.dart';
part 'public_media_socket_attempt.dart';

typedef MediaSocketStarter = Future<ConnectionTask<Socket>> Function(
  InternetAddress address,
  int port,
);

typedef MediaRawSocketStarter = Future<MediaRawSocketTask> Function(
  InternetAddress address,
  int port,
);

abstract interface class MediaRawSocketTask {
  Future<RawSocket> get socket;

  void cancel();

  static Future<MediaRawSocketTask> startConnect(
    InternetAddress address,
    int port,
  ) async {
    final task = await RawSocket.startConnect(address, port);
    return _IoMediaRawSocketTask(task);
  }
}

final class _IoMediaRawSocketTask implements MediaRawSocketTask {
  const _IoMediaRawSocketTask(this._task);

  final ConnectionTask<RawSocket> _task;

  @override
  Future<RawSocket> get socket => _task.socket;

  @override
  void cancel() => _task.cancel();
}

class PublicMediaHttpClientConfig {
  const PublicMediaHttpClientConfig({
    this.startConnect,
    this.startRawConnect,
    this.connectionTimeout = const Duration(seconds: 10),
    this.securityContext,
  });

  final MediaSocketStarter? startConnect;
  final MediaRawSocketStarter? startRawConnect;
  final Duration connectionTimeout;
  final SecurityContext? securityContext;
}

http.Client createPublicMediaHttpClient(
  PublicMediaAddressResolver resolver, {
  PublicMediaHttpClientConfig config = const PublicMediaHttpClientConfig(),
}) {
  final factory = _PublicMediaConnectionFactory(resolver, config);
  final client = HttpClient();
  client.connectionTimeout = config.connectionTimeout;
  client.findProxy = (_) => 'DIRECT';
  client.connectionFactory = factory.connect;
  return IOClient(client);
}
