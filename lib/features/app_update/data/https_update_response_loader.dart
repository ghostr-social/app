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

  Future<http.StreamedResponse> load(Uri initial) async {
    var uri = initial;
    var redirects = 0;
    while (true) {
      final response = await _send(uri);
      if (!_redirectStatuses.contains(response.statusCode)) return response;
      if (redirects >= maximumRedirects) throw _failure();
      redirects += 1;
      uri = await _redirect(response, uri);
    }
  }

  Future<http.StreamedResponse> _send(Uri uri) {
    final request = http.Request('GET', uri)..followRedirects = false;
    return _client.send(request).timeout(timeout);
  }

  Future<Uri> _redirect(http.StreamedResponse response, Uri current) async {
    final location = response.headers[HttpHeaders.locationHeader];
    await response.stream.drain<void>();
    if (location == null) throw _failure();
    final target = current.resolve(location);
    if (target.scheme != 'https' ||
        target.host.isEmpty ||
        target.userInfo.isNotEmpty) {
      throw _failure();
    }
    return target;
  }

  AppFailure _failure() => const AppFailure('Could not download the update.');
}
