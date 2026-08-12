import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/domain/profile_image_picker_port.dart';
import 'package:ghostr/features/profile/domain/profile_image_workflow.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/domain/selected_profile_image.dart';
import 'package:ghostr/features/profile/presentation/profile_edit_cubit.dart';

import '../support/fake_profile_image_services.dart';
import '../support/fake_profile_metadata_repository.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'profile edit ignores save while picture selection is pending',
    () async {
      final picker = _PendingProfileImagePicker();
      final repository = FakeProfileMetadataRepository();
      final cubit = ProfileEditCubit(
        repository,
        sampleSession().identity,
        ProfileImageWorkflow(picker, FakeProfileImageUploader()),
      );
      addTearDown(cubit.close);
      final selection = cubit.selectPicture();

      expect(cubit.state, isA<ProfileEditSelectingPicture>());
      await cubit.save(
        ProfileMetadata.parse(displayName: 'Nora', handle: 'nora'),
      );

      expect(cubit.state, isA<ProfileEditSelectingPicture>());
      expect(repository.savedMetadata, isNull);
      picker.release.complete(null);
      await selection;
      expect(cubit.state, isA<ProfileEditIdle>());
    },
  );
}

final class _PendingProfileImagePicker implements ProfileImagePickerPort {
  final release = Completer<SelectedProfileImage?>();

  @override
  Future<SelectedProfileImage?> pickFromGallery() => release.future;
}
