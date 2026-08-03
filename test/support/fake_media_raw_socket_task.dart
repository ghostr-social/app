import 'dart:io';

import 'package:ghostr/platform/network/public_media_http_client.dart';

final class FakeMediaRawSocketTask implements MediaRawSocketTask {
  const FakeMediaRawSocketTask(this.socket, this._onCancel);

  @override
  final Future<RawSocket> socket;

  final void Function() _onCancel;

  @override
  void cancel() => _onCancel();
}
