import 'dart:convert';
import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/app_update/domain/release_artifact.dart';
import 'package:ghostr/features/app_update/domain/update_package_downloader.dart';

final class UpdateFileWriter {
  UpdateFileWriter(File file, this._artifact) : _output = file.openWrite() {
    _digest = sha256.startChunkedConversion(_capture);
  }

  final ReleaseArtifact _artifact;
  final IOSink _output;
  final _DigestCapture _capture = _DigestCapture();
  late final ByteConversionSink _digest;
  var _received = 0;

  UpdateDownloadProgress add(List<int> chunk) {
    _received += chunk.length;
    if (_received > _artifact.sizeBytes) throw updateDownloadFailure();
    _output.add(chunk);
    _digest.add(chunk);
    return UpdateDownloadProgress(
      bytes: _received,
      totalBytes: _artifact.sizeBytes,
    );
  }

  Future<void> flush() => _output.flush();

  Future<void> close() async {
    _digest.close();
    await _output.close();
  }

  bool get matches {
    return _received == _artifact.sizeBytes &&
        _capture.value.toString() == _artifact.sha256.value;
  }
}

final class _DigestCapture implements Sink<Digest> {
  late Digest value;

  @override
  void add(Digest data) => value = data;

  @override
  void close() {}
}

AppFailure updateDownloadFailure() {
  return const AppFailure('Could not download the update.');
}
