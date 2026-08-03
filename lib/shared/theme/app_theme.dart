import 'package:flutter/material.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

ThemeData buildAppTheme() {
  return ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    colorScheme: _colorScheme(),
    scaffoldBackgroundColor: AppPalette.night,
    dividerColor: AppPalette.divider,
    snackBarTheme: _snackBarTheme,
    textTheme: _textTheme,
    inputDecorationTheme: _inputDecorationTheme(),
    elevatedButtonTheme: _elevatedButtonTheme(),
  );
}

const _snackBarTheme = SnackBarThemeData(behavior: SnackBarBehavior.floating);
const _textTheme = TextTheme(
  headlineMedium: TextStyle(fontWeight: FontWeight.w700),
  titleLarge: TextStyle(fontWeight: FontWeight.w700),
  titleMedium: TextStyle(fontWeight: FontWeight.w700),
  bodyLarge: TextStyle(height: 1.3),
  bodyMedium: TextStyle(height: 1.3),
);

ColorScheme _colorScheme() {
  return ColorScheme.fromSeed(
    seedColor: AppPalette.accentRed,
    brightness: Brightness.dark,
    surface: AppPalette.surface,
  ).copyWith(
    primary: AppPalette.accentRed,
    secondary: AppPalette.accentBlue,
    surface: AppPalette.surface,
    onSurface: AppPalette.foreground,
    onPrimary: AppPalette.foreground,
  );
}

InputDecorationTheme _inputDecorationTheme() {
  return InputDecorationTheme(
    filled: true,
    fillColor: AppPalette.altSurface,
    border: OutlineInputBorder(
      borderRadius: BorderRadius.circular(AppRadius.control),
      borderSide: BorderSide.none,
    ),
  );
}

ElevatedButtonThemeData _elevatedButtonTheme() {
  return ElevatedButtonThemeData(
    style: ElevatedButton.styleFrom(
      backgroundColor: AppPalette.accentRed,
      foregroundColor: AppPalette.foreground,
      minimumSize: const Size.fromHeight(52),
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(AppRadius.control),
      ),
    ),
  );
}
