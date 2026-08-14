import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/features/app_update/data/https_update_response_loader.dart';
import 'package:ghostr/features/app_update/data/stable_release_parser.dart';
import 'package:ghostr/features/app_update/domain/app_release_catalog.dart';
import 'package:ghostr/features/app_update/domain/stable_release.dart';
import 'package:http/http.dart' as http;

final class HttpAppReleaseCatalog implements AppReleaseCatalog {
  const HttpAppReleaseCatalog(
    this._client, {
    StableReleaseParser parser = const StableReleaseParser(),
    Duration timeout = const Duration(seconds: 15),
    Uri? endpoint,
  }) : _parser = parser,
       _timeout = timeout,
       _endpoint = endpoint;

  static final stableEndpoint = Uri.https(
    'github.com',
    '/ghostr-social/app/releases/latest/download/stable.json',
  );
  static final fallbackEndpoint = Uri.https(
    'ghostr-social.github.io',
    '/stable.json',
  );
  static const maximumMetadataBytes = 64 * 1024;

  final http.Client _client;
  final StableReleaseParser _parser;
  final Duration _timeout;
  final Uri? _endpoint;

  @override
  Future<StableRelease> fetchStableRelease() async {
    final endpoint = _endpoint;
    if (endpoint != null) return _fetchSafely(endpoint);
    Uint8List bytes;
    try {
      bytes = await _load(stableEndpoint);
    } on Object catch (error, stackTrace) {
      _logPrimaryFailure(error, stackTrace);
      return _fetchSafely(fallbackEndpoint);
    }
    return _parseSafely(bytes);
  }

  Future<StableRelease> _fetchSafely(Uri endpoint) async {
    Uint8List bytes;
    try {
      bytes = await _load(endpoint);
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'ghostr.app-update.catalog',
        message: 'Could not check for updates.',
        error: error,
        stackTrace: stackTrace,
      );
    }
    return _parseSafely(bytes);
  }

  Future<Uint8List> _load(Uri endpoint) async {
    final response = await _send(endpoint);
    return _readBounded(response);
  }

  StableRelease _parseSafely(Uint8List bytes) {
    try {
      return _parser.parse(utf8.decode(bytes));
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'ghostr.app-update.catalog',
        message: 'Could not check for updates.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }

  void _logPrimaryFailure(Object error, StackTrace stackTrace) {
    logBoundaryFailure(
      source: 'ghostr.app-update.catalog.primary',
      message: 'The primary update catalog was unavailable.',
      error: error,
      stackTrace: stackTrace,
    );
  }

  Future<http.StreamedResponse> _send(Uri endpoint) async {
    final response = await HttpsUpdateResponseLoader(_client, timeout: _timeout)
        .load(
          endpoint,
          headers: const {'cache-control': 'no-cache', 'pragma': 'no-cache'},
        );
    final length = response.contentLength;
    if (response.statusCode != HttpStatus.ok ||
        (length != null && length > maximumMetadataBytes)) {
      throw const AppFailure('Could not check for updates.');
    }
    return response;
  }

  Future<Uint8List> _readBounded(http.StreamedResponse response) async {
    final output = BytesBuilder(copy: false);
    await for (final chunk in response.stream.timeout(_timeout)) {
      output.add(chunk);
      if (output.length > maximumMetadataBytes) {
        throw const AppFailure('Could not check for updates.');
      }
    }
    return output.takeBytes();
  }
}
