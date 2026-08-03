part of 'public_media_http_client.dart';

class _RawSecureSocketConsumer implements StreamConsumer<List<int>> {
  const _RawSecureSocketConsumer(this._socket);

  final RawSecureSocketAdapter _socket;

  @override
  Future<void> addStream(Stream<List<int>> stream) async {
    await for (final bytes in stream) {
      await _socket._writeAll(bytes);
    }
  }

  @override
  Future<void> close() => _socket._closeOutput();
}
