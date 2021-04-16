import 'dart:async';

import 'package:flutter/services.dart';

class EsseCore {
  static const MethodChannel _channel = const MethodChannel('esse_core');

  static Future<String> get platformVersion async {
    final String version = await _channel.invokeMethod('getPlatformVersion');
    return version;
  }

  static Future<void> daemon(String path) async {
    final String version =
        await _channel.invokeMethod('daemon', {'path': path});
    print("over daemon: " + version);
  }
}
