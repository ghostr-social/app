import 'dart:async';
import 'dart:io';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/features/video_inventory/domain/video_download_limit_exceeded.dart';
import 'package:ghostr/features/video_inventory/domain/video_file_downloader.dart';
import 'package:ghostr/features/video_inventory/domain/media_url_policy.dart';
import 'package:ghostr/platform/media/video_download_timeouts.dart';
import 'package:http/http.dart' as http;

part 'http_video_download_control.dart';

class HttpVideoFileDownloader implements VideoFileDownloader {
  const HttpVideoFileDownloader(
    this._client,
    this._urlPolicy, {
    VideoDownloadTimeouts timeouts = VideoDownloadTimeouts.defaults,
  }) : _timeouts = timeouts;

  final http.Client _client;
  final MediaUrlPolicy _urlPolicy;
  final VideoDownloadTimeouts _timeouts;

  @override
  Future<void> download(
    Uri source,
    String destinationPath, {
    required int maxBytes,
    Duration? totalTimeout,
  }) async {
    final deadline = _RequestControl(totalTimeout ?? _timeouts.total);
    try {
      await _download(source, destinationPath, maxBytes, deadline);
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'ghostr.media.http-cache',
        message: 'The video could not be cached.',
        error: error,
        stackTrace: stackTrace,
      );
    } finally {
      deadline.close();
    }
  }

  Future<void> _download(
    Uri source,
    String destinationPath,
    int maxBytes,
    _RequestControl deadline,
  ) async {
    final exchange = await _send(source, 0, deadline);
    try {
      await _requireSuccess(exchange);
      await exchange.untilAborted(
        _writeWithinLimit(exchange.stream, destinationPath, maxBytes),
      );
    } finally {
      exchange.close();
    }
  }

  Future<_DownloadResponse> _send(
    Uri source,
    int redirects,
    _RequestControl deadline,
  ) async {
    await deadline.untilAborted(_urlPolicy.validate(source));
    deadline.requireActive();
    final exchange = await _request(source, deadline.aborted);
    late final Uri? target;
    try {
      target = _redirectTarget(source, exchange);
    } on Object {
      await exchange.discard();
      rethrow;
    }
    if (target == null) return exchange;
    await exchange.discard();
    if (redirects >= 5) {
      throw const AppFailure('Video download exceeded the redirect limit.');
    }
    return _send(target, redirects + 1, deadline);
  }

  Future<_DownloadResponse> _request(
    Uri source,
    Future<void> deadline,
  ) async {
    final control = _RequestControl(_timeouts.headers, deadline);
    final request = http.AbortableRequest(
      'GET',
      source,
      abortTrigger: control.aborted,
    )..followRedirects = false;
    try {
      final response = await control.untilAborted(
        _client.send(request),
      );
      control.cancelTimer();
      return _DownloadResponse(response, control, _timeouts.idle);
    } on Object {
      control.close();
      rethrow;
    }
  }

  Uri? _redirectTarget(Uri source, _DownloadResponse exchange) {
    final response = exchange.response;
    final location = response.headers['location'];
    if (!_isRedirect(response.statusCode) || location == null) return null;
    return source.resolve(location);
  }

  bool _isRedirect(int statusCode) {
    return statusCode == 301 ||
        statusCode == 302 ||
        statusCode == 303 ||
        statusCode == 307 ||
        statusCode == 308;
  }

  Future<void> _requireSuccess(_DownloadResponse exchange) async {
    final statusCode = exchange.response.statusCode;
    if (statusCode == HttpStatus.ok) return;
    await exchange.discard();
    throw AppFailure('Video download failed ($statusCode).');
  }

  Future<void> _writeWithinLimit(
    Stream<List<int>> stream,
    String destinationPath,
    int maxBytes,
  ) async {
    final sink = File(destinationPath).openWrite();
    var written = 0;
    try {
      await for (final chunk in stream) {
        if (written + chunk.length > maxBytes) {
          throw const VideoDownloadLimitExceeded();
        }
        sink.add(chunk);
        written += chunk.length;
      }
    } finally {
      await sink.close();
    }
  }
}
