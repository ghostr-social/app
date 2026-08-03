import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/shared/widgets/profile_avatar.dart';

void main() {
  test('resolves remote avatar images only for absolute http(s) URLs', () {
    expect(remoteAvatarImage(null), isNull);
    expect(remoteAvatarImage(''), isNull);
    expect(remoteAvatarImage('notaurl'), isNull);
    expect(remoteAvatarImage('ftp://x/y'), isNull);

    final image = remoteAvatarImage('https://example.com/a.png');
    expect(image, isA<NetworkImage>());
    expect((image as NetworkImage).url, 'https://example.com/a.png');
  });
}
