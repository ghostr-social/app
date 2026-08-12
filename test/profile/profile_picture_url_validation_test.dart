import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';

void main() {
  test(
    'picture is optional and otherwise requires an absolute HTTP(S) URL',
    () {
      final secure = ProfileMetadata.parse(
        displayName: 'Nora',
        handle: 'nora',
        pictureUrl: ' https://cdn.example/nora.png ',
      );
      final absent = ProfileMetadata.parse(
        displayName: 'Nora',
        handle: 'nora',
        pictureUrl: '  ',
      );
      final insecure = ProfileMetadata.parse(
        displayName: 'Nora',
        handle: 'nora',
        pictureUrl: 'http://localhost/nora.png',
      );

      expect(secure.pictureUrl?.value, 'https://cdn.example/nora.png');
      expect(insecure.pictureUrl?.value, 'http://localhost/nora.png');
      expect(absent.pictureUrl, isNull);
      final invalidUrls = [
        'avatar.png',
        '/avatar.png',
        'https:avatar.png',
        'http:///avatar.png',
        'ftp://example/a',
      ];
      for (final invalid in invalidUrls) {
        expect(
          () => ProfileMetadata.parse(
            displayName: 'Nora',
            handle: 'nora',
            pictureUrl: invalid,
          ),
          throwsFormatException,
        );
      }
    },
  );

  test('picture URL value object parses validated web locations', () {
    expect(
      ProfilePictureUrl.parse(' https://cdn.example/avatar.png ').value,
      'https://cdn.example/avatar.png',
    );
    expect(() => ProfilePictureUrl.parse(' '), throwsFormatException);
  });
}
