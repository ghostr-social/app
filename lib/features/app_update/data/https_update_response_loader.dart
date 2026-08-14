import 'dart:async';
import 'dart:io';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:http/http.dart' as http;

final class HttpsUpdateResponseLoader {
  const HttpsUpdateResponseLoader(
    this._client, {
    this.timeout = const Duration(seconds: 15),
    this.maximumRedirects = 5,
  });

  static const _redirectStatuses = {
    HttpStatus.movedPermanently,
    HttpStatus.found,
    HttpStatus.seeOther,
    HttpStatus.temporaryRedirect,
    HttpStatus.permanentRedirect,
  };

  final http.Client _client;
  final Duration timeout;
  final int maximumRedirects;

  Future<http.StreamedResponse> load(
    Uri initial, {
    Map<String, String> headers = const {},
  }) async {
    if (!_isSafe(initial)) throw _failure();
    var uri = initial;
    var redirects = 0;
    while (true) {
      final response = await _send(uri, headers);
      if (!_redirectStatuses.contains(response.statusCode)) return response;
      if (redirects >= maximumRedirects) {
        await response.stream.timeout(timeout).drain<void>();
        throw _failure();
      }
      redirects += 1;
      uri = await _redirect(response, uri);
    }
  }

  Future<http.StreamedResponse> _send(Uri uri, Map<String, String> headers) {
    final request = http.Request('GET', uri)
      ..followRedirects = false
      ..headers.addAll(headers);
    return _client.send(request).timeout(timeout);
  }

  Future<Uri> _redirect(http.StreamedResponse response, Uri current) async {
    final location = response.headers[HttpHeaders.locationHeader];
    await response.stream.timeout(timeout).drain<void>();
    if (location == null) throw _failure();
    final target = current.resolve(location);
    if (!_isSafe(target)) throw _failure();
    return target;
  }

  bool _isSafe(Uri uri) {
    return uri.scheme == 'https' && uri.host.isNotEmpty && uri.userInfo.isEmpty;
  }

  AppFailure _failure() => const AppFailure('Could not download the update.');
}
