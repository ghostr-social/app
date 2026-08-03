import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/home_tab.dart';

void main() {
  test('defines the stable navigation label and icon for every home tab', () {
    expect(
      HomeTab.values.map((tab) => (tab.label, tab.icon)),
      const [
        ('Home', Icons.home_rounded),
        ('Search', Icons.search_rounded),
        ('Create', Icons.add_box_rounded),
        ('Activity', Icons.notifications_rounded),
        ('Profile', Icons.person_rounded),
      ],
    );
  });
}
