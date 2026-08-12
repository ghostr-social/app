import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
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
    try {
      final response = await _send();
      final bytes = await _readBounded(response);
      return _parser.parse(utf8.decode(bytes));
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'ghostr.app-update.catalog',
        message: 'Could not check for updates.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }

  Future<http.StreamedResponse> _send() async {
    final request = http.Request('GET', _endpoint ?? stableEndpoint)
      ..followRedirects = false;
    final response = await _client.send(request).timeout(_timeout);
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
