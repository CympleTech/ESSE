// device_info_plus: any no-need.
import 'dart:io' show Platform;
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/services.dart' show PlatformException;
import 'package:device_info_plus/device_info_plus.dart';

Future<List<String>> deviceInfo() async {
  final DeviceInfoPlugin deviceInfoPlugin = DeviceInfoPlugin();

  if (kIsWeb) {
    try {
      final data = await deviceInfoPlugin.webBrowserInfo;
      return [
        "Web",
        data.platform!
      ];
    } catch (_e) {
      return ["Web", "Web Unknown"];
    }
  } else {
    if (Platform.isAndroid) {
      try {
        final data = await deviceInfoPlugin.androidInfo;
        return [
          data.brand!,
          data.id!,
        ];
      } catch (_e) {
        return ["Android", "Android Unknown"];
      }
    } else if (Platform.isIOS) {
      try {
        final data = await deviceInfoPlugin.iosInfo;
        return [
          data.name!,
          data.utsname.machine!,
        ];
      } catch (_e) {
        return ["IOS", "IOS Unknown"];
      }
    } else if (Platform.isLinux) {
      try {
        final data = await deviceInfoPlugin.linuxInfo;
        return [
          data.name,
          data.prettyName,
        ];
      } catch (_e) {
        return ["Linux", "Linux Unknown"];
      }
    } else if (Platform.isMacOS) {
      try {
        final data = await deviceInfoPlugin.macOsInfo;
        return [
          data.hostName,
          data.computerName,
        ];
      } catch (_e) {
        return ["MacOS", "MacOS Unknown"];
      }
    } else if (Platform.isWindows) {
      try {
        final data = await deviceInfoPlugin.windowsInfo;
        return [
          'Windows',
          data.computerName,
        ];
      } catch (_e) {
        return ["Windows", "Windows Unknown"];
      }
    }
  }
  return [
    "Unknown", "Unknown"
  ];
}
