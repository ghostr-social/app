import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/search_screen.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/search_screen_harness.dart';

void main() {
  testWidgets('an externally triggered search fills the query field',
      (tester) async {
    final repository = FakeVideoCatalogRepository(forYouFeed: [samplePost()]);
    await tester.pumpWidget(searchScreenHarness(repository));

    final cubit = tester.element(find.byType(SearchScreen)).read<SearchCubit>();
    await cubit.search('#tag');
    await tester.pumpAndSettle();

    final field = tester.widget<TextField>(find.byType(TextField));
    expect(field.controller?.text, '#tag');
  });
}
