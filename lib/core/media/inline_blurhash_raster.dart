part of 'inline_blurhash.dart';

typedef _LinearColor = ({double red, double green, double blue});

final class _BlurHashDecoder {
  const _BlurHashDecoder(this.descriptor, this.width, this.height);

  final InlineBlurHash descriptor;
  final int width;
  final int height;

  List<int> decode() {
    final coefficients = _coefficients();
    return List<int>.generate(
      width * height,
      (index) => _pixel(index, coefficients),
      growable: false,
    );
  }

  List<_LinearColor> _coefficients() {
    final count =
        descriptor.horizontalComponents * descriptor.verticalComponents;
    final maximum = (_decode83(descriptor.encoded, 1, 1) + 1) / 166;
    return List<_LinearColor>.generate(
      count,
      (index) => index == 0 ? _dc() : _ac(index, maximum),
      growable: false,
    );
  }

  _LinearColor _dc() {
    final value = _decode83(descriptor.encoded, 2, 4);
    return (
      red: _srgbToLinear(value >> 16),
      green: _srgbToLinear((value >> 8) & 255),
      blue: _srgbToLinear(value & 255),
    );
  }

  _LinearColor _ac(int index, double maximum) {
    final value = _decode83(descriptor.encoded, 4 + index * 2, 2);
    return (
      red: _signedSquare((value ~/ (19 * 19) - 9) / 9) * maximum,
      green: _signedSquare((value ~/ 19 % 19 - 9) / 9) * maximum,
      blue: _signedSquare((value % 19 - 9) / 9) * maximum,
    );
  }

  int _pixel(int index, List<_LinearColor> coefficients) {
    final x = index % width;
    final y = index ~/ width;
    final linear = _linearPixel(x, y, coefficients);
    final red = _linearToSrgb(linear.red);
    final green = _linearToSrgb(linear.green);
    final blue = _linearToSrgb(linear.blue);
    return 0xff000000 | red << 16 | green << 8 | blue;
  }

  _LinearColor _linearPixel(int x, int y, List<_LinearColor> coefficients) {
    var red = 0.0;
    var green = 0.0;
    var blue = 0.0;
    for (var row = 0; row < descriptor.verticalComponents; row += 1) {
      for (
        var column = 0;
        column < descriptor.horizontalComponents;
        column += 1
      ) {
        final basis = _basis(x, y, column, row);
        final color =
            coefficients[row * descriptor.horizontalComponents + column];
        red += color.red * basis;
        green += color.green * basis;
        blue += color.blue * basis;
      }
    }
    return (red: red, green: green, blue: blue);
  }

  double _basis(int x, int y, int column, int row) {
    final horizontal = math.cos(math.pi * x * column / width);
    final vertical = math.cos(math.pi * y * row / height);
    return horizontal * vertical;
  }
}

double _signedSquare(double value) {
  return value.sign * math.pow(value.abs(), 2).toDouble();
}

double _srgbToLinear(int value) {
  final normalized = value / 255;
  if (normalized <= 0.04045) return normalized / 12.92;
  return math.pow((normalized + 0.055) / 1.055, 2.4).toDouble();
}

int _linearToSrgb(double value) {
  final normalized = value.clamp(0.0, 1.0);
  final converted = normalized <= 0.0031308
      ? normalized * 12.92
      : 1.055 * math.pow(normalized, 1 / 2.4) - 0.055;
  return (converted * 255 + 0.5).toInt();
}
