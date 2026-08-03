import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/network/public_media_address_resolver.dart';
import 'package:ghostr/platform/network/public_media_http_client.dart';

void main() {
  test('uses a later address when the first TLS handshake stalls', () async {
    final stalled = await ServerSocket.bind(InternetAddress.loopbackIPv4, 0);
    addTearDown(stalled.close);
    final stalledPeerClosed = Completer<void>();
    stalled.listen((socket) {
      socket.listen((_) {}, onDone: stalledPeerClosed.complete);
    });
    const certificate = 'test/support/certificates/media_example_cert.pem';
    const privateKey = 'test/support/certificates/media_example_key.pem';
    final serverContext = SecurityContext()
      ..useCertificateChain(certificate)
      ..usePrivateKey(privateKey);
    final healthy = await HttpServer.bindSecure(
      InternetAddress.loopbackIPv4,
      0,
      serverContext,
    );
    addTearDown(() => healthy.close(force: true));
    healthy.listen((request) async {
      request.response.write('ok');
      await request.response.close();
    });
    final clientContext = SecurityContext(withTrustedRoots: false)
      ..setTrustedCertificates(certificate);
    final resolver = PublicMediaAddressResolver(
      lookup: (_) async => [
        InternetAddress('8.8.8.8'),
        InternetAddress('1.1.1.1'),
      ],
    );
    final client = createPublicMediaHttpClient(
      resolver,
      config: PublicMediaHttpClientConfig(
        startRawConnect: (address, _) => MediaRawSocketTask.startConnect(
          InternetAddress.loopbackIPv4,
          address.address == '8.8.8.8' ? stalled.port : healthy.port,
        ),
        securityContext: clientContext,
        connectionTimeout: const Duration(milliseconds: 900),
      ),
    );
    addTearDown(client.close);

    final response = await client.get(
      Uri.parse('https://media.example/video.mp4'),
    );

    expect(response.body, 'ok');
    await stalledPeerClosed.future.timeout(const Duration(seconds: 1));
  });
}
