import 'dart:io';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/platform/sharing/gateway_video_file_downloader.dart';
import 'package:http/http.dart' as http;

final class HttpVideoFileTransfer implements VideoFileTransfer {
  const HttpVideoFileTransfer(this._client);

  final http.Client _client;

  @override
  Future<void> transfer(Uri source, String destination) async {
    final partial = File('$destination.partial');
    try {
      final response = await _client.send(http.Request('GET', source));
      _accept(response);
      await partial.parent.create(recursive: true);
      await response.stream.pipe(partial.openWrite());
      await _validate(partial, response.contentLength);
      await _install(partial, File(destination));
    } on AppFailure {
      await _remove(partial);
      rethrow;
    } on Object catch (error, stackTrace) {
      await _remove(partial);
      throw translatedBoundaryFailure(
        source: 'ghostr.video.share.download',
        message: 'Could not download this video.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}

void _accept(http.StreamedResponse response) {
  if (response.statusCode != HttpStatus.ok) {
    throw const AppFailure('Could not download this video.');
  }
}

Future<void> _install(File partial, File destination) async {
  if (await destination.exists()) await destination.delete();
  await partial.rename(destination.path);
}

Future<void> _validate(File file, int? expectedBytes) async {
  final actualBytes = await file.length();
  if (expectedBytes == null ||
      expectedBytes < 1 ||
      actualBytes != expectedBytes) {
    throw const AppFailure('Could not download this video.');
  }
}

Future<void> _remove(File file) async {
  if (await file.exists()) await file.delete();
}
