import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/shared/widgets/profile_avatar.dart';

void main() {
  testWidgets('falls back to initials when the avatar image fails to load',
      (tester) async {
    await tester.pumpWidget(const MaterialApp(
      home: Scaffold(
        body: ProfileAvatar(
          initials: 'NR',
          avatarUrl: 'https://example.com/avatar.png',
        ),
      ),
    ));
    await tester.pump();

    expect(find.text('NR'), findsOneWidget);
  });
}
