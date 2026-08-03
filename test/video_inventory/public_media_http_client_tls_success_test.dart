import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/network/public_media_address_resolver.dart';
import 'package:ghostr/platform/network/public_media_http_client.dart';

void main() {
  test('downloads HTTPS media through a validated pinned address', () async {
    const certificate = 'test/support/certificates/media_example_cert.pem';
    const privateKey = 'test/support/certificates/media_example_key.pem';
    final serverContext = SecurityContext()
      ..useCertificateChain(certificate)
      ..usePrivateKey(privateKey);
    final server = await HttpServer.bindSecure(
      InternetAddress.loopbackIPv4,
      0,
      serverContext,
    );
    addTearDown(() => server.close(force: true));
    server.listen((request) async {
      request.response.write('abc');
      await request.response.close();
    });
    final clientContext = SecurityContext(withTrustedRoots: false)
      ..setTrustedCertificates(certificate);
    final resolver = PublicMediaAddressResolver(
      lookup: (_) async => [InternetAddress('8.8.8.8')],
    );
    final client = createPublicMediaHttpClient(
      resolver,
      config: PublicMediaHttpClientConfig(
        startRawConnect: (_, __) => MediaRawSocketTask.startConnect(
          InternetAddress.loopbackIPv4,
          server.port,
        ),
        securityContext: clientContext,
      ),
    );
    addTearDown(client.close);

    final response = await client.get(
      Uri.parse('https://media.example/video.mp4'),
    );

    expect(response.statusCode, HttpStatus.ok);
    expect(response.body, 'abc');
  });
}
