import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/presentation/comment_composer.dart';

void main() {
  testWidgets('disables comment editing while a post is pending',
      (tester) async {
    final controller = TextEditingController(text: 'Pending');
    final focusNode = FocusNode();
    addTearDown(controller.dispose);
    addTearDown(focusNode.dispose);

    await tester.pumpWidget(MaterialApp(
      home: Scaffold(
        body: CommentComposer(
          model: CommentComposerModel(
            controller: controller,
            focusNode: focusNode,
            replyTo: null,
            isPosting: true,
          ),
          onChanged: () {},
          onPublish: () {},
        ),
      ),
    ));

    expect(tester.widget<TextField>(find.byType(TextField)).enabled, isFalse);
    expect(
        tester.widget<IconButton>(find.byType(IconButton)).onPressed, isNull);
  });
}
