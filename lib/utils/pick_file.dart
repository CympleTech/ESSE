import 'dart:io' show Platform;

import 'package:file_picker/file_picker.dart';
import 'package:file_selector/file_selector.dart';

Future<String?> pickFile() async {
  if (Platform.isLinux || Platform.isMacOS || Platform.isWindows) {
    final XTypeGroup typeGroup = XTypeGroup(label: 'files');
    final file = await openFile(acceptedTypeGroups: [typeGroup]);
    if (file != null) {
      return file.path;
    }
  } else if (Platform.isAndroid || Platform.isIOS) {
    FilePickerResult? result = await FilePicker.platform.pickFiles();

    if(result != null) {
      return result.files.single.path;
    }
  }

  return null;
}
