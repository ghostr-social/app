import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/domain/profile_image_workflow.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';

import '../support/fake_profile_image_services.dart';

void main() {
  test('no image selection leaves profile metadata unchanged', () async {
    final workflow = fakeProfileImages();
    final metadata = ProfileMetadata.parse(displayName: 'Nora', handle: 'nora');

    final selected = await workflow.select();
    final resolved = await workflow.resolve(metadata, selected);

    expect(selected, isNull);
    expect(resolved, same(metadata));
    final disabled = ProfileImageWorkflow.disabled();
    expect(await disabled.select(), isNull);
  });
}
