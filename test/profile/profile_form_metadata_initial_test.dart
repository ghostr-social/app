import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/presentation/profile_metadata_form_screen.dart';

void main() {
  test('prefills every form value from public profile metadata', () {
    final metadata = ProfileMetadata.parse(
      displayName: 'Nora Relay',
      handle: '@nora',
      pictureUrl: 'https://cdn.example/nora.png',
    );

    final initial = ProfileFormInitial.fromMetadata(metadata);

    expect(initial.displayName, 'Nora Relay');
    expect(initial.handle, 'nora');
    expect(initial.pictureUrl, 'https://cdn.example/nora.png');
  });
}
