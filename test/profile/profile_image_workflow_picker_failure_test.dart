import 'package:flutter_test/flutter_test.dart';

import '../support/fake_profile_image_services.dart';

void main() {
  test('image selection preserves picker failures for the caller', () async {
    final picker = FakeProfileImagePicker()..failure = StateError('denied');
    final workflow = fakeProfileImages(picker: picker);

    await expectLater(workflow.select(), throwsStateError);
    expect(picker.pickCount, 1);
  });
}
