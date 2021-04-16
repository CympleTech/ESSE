import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:esse_core/esse_core.dart';

void main() {
  const MethodChannel channel = MethodChannel('esse_core');

  TestWidgetsFlutterBinding.ensureInitialized();

  setUp(() {
    channel.setMockMethodCallHandler((MethodCall methodCall) async {
      return '42';
    });
  });

  tearDown(() {
    channel.setMockMethodCallHandler(null);
  });

  test('getPlatformVersion', () async {
    expect(await EsseCore.platformVersion, '42');
  });
}
