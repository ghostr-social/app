import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/domain/profile_image_workflow.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';

import '../support/fake_profile_image_services.dart';

void main() {
  test('disabled image workflow rejects an attempted upload', () async {
    final metadata = ProfileMetadata.parse(displayName: 'Nora', handle: 'nora');

    // Deliberately non-const so the constructor executes at runtime.
    final upload = ProfileImageWorkflow.disabled().resolve(
      metadata,
      sampleProfileImage(),
    );

    await expectLater(
      upload,
      throwsA(
        isA<StateError>().having(
          (error) => error.message,
          'message',
          'Profile image upload is unavailable.',
        ),
      ),
    );
  });
}
