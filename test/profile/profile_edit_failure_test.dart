import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/presentation/profile_edit_cubit.dart';

import '../support/fake_profile_metadata_repository.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'profile edit exposes a safe save failure and remains retryable',
    () async {
      final session = sampleSession();
      final repository = FakeProfileMetadataRepository()
        ..saveFailure = const AppFailure('No relay accepted this profile.');
      final cubit = ProfileEditCubit(repository, session.identity);
      addTearDown(cubit.close);
      final metadata = ProfileMetadata.parse(
        displayName: 'Nora Updated',
        handle: 'nora',
      );

      await cubit.save(metadata);

      expect(cubit.state, isA<ProfileEditFailure>());
      expect(cubit.state.message, 'No relay accepted this profile.');
      repository.saveFailure = null;
      await cubit.save(metadata);
      expect(cubit.state, isA<ProfileEditSaved>());
    },
  );
}
