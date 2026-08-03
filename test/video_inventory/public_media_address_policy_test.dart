import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/network/public_media_address_resolver.dart';

void main() {
  test('only public HTTP media destinations pass validation', () async {
    final rejected = File(
      'test/support/public_media_private_addresses.txt',
    ).readAsLinesSync();
    for (final value in rejected) {
      final policy = PublicMediaAddressResolver(
        lookup: (_) async => [InternetAddress(value)],
      );
      await expectLater(
        policy.validate(Uri.parse('https://media.test/video.mp4')),
        throwsA(isA<AppFailure>()),
        reason: value,
      );
    }

    const accepted = [
      '8.8.8.8',
      '93.184.216.34',
      '::ffff:8.8.8.8',
      '64:ff9b::808:808',
      '2001:4860:4860::8888',
      '2606:4700:4700::1111',
    ];
    for (final value in accepted) {
      final policy = PublicMediaAddressResolver(
        lookup: (_) async => [InternetAddress(value)],
      );
      await policy.validate(Uri.parse('https://media.test/video.mp4'));
    }

    final unused = PublicMediaAddressResolver(
      lookup: (_) => throw StateError('must not resolve invalid URLs'),
    );
    for (final source in [
      Uri.parse('file:///tmp/video.mp4'),
      Uri.parse('https://user@media.test/video.mp4'),
    ]) {
      await expectLater(unused.validate(source), throwsA(isA<AppFailure>()));
    }
  });
}
