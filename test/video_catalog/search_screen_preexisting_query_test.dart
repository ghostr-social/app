import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/search_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/trending_hashtags_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('shows the query of a search finished before the screen opened',
      (tester) async {
    final repository = FakeVideoCatalogRepository(forYouFeed: [samplePost()]);
    final cubit = SearchCubit(repository);
    addTearDown(cubit.close);
    await cubit.search('#trend');

    await tester.pumpWidget(MaterialApp(
      home: MultiBlocProvider(
        providers: [
          BlocProvider.value(value: cubit),
          BlocProvider(create: (_) => TrendingHashtagsCubit(repository)),
        ],
        child: Scaffold(
          body: SearchScreen(onOpenProfile: (_) {}, onOpenFeed: (_) {}),
        ),
      ),
    ));
    await tester.pumpAndSettle();

    final field = tester.widget<TextField>(find.byType(TextField));
    expect(field.controller?.text, '#trend');
  });
}
