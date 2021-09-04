import 'dart:io' show Platform;

import 'package:image_picker/image_picker.dart';
import 'package:file_selector_platform_interface/file_selector_platform_interface.dart';

Future<String?> pickImage() async {
  if (Platform.isLinux || Platform.isMacOS || Platform.isWindows) {
    final XTypeGroup typeGroup = XTypeGroup(
      label: 'images',
      extensions: ['jpg', 'png'],
    );
    final List<XFile>? files = await FileSelectorPlatform.instance.openFiles(acceptedTypeGroups: [typeGroup]);
    if (files != null && files.length > 0) {
      final XFile file = files[0];
      return file.path;
    }
  } else {
    final pickedFile = await ImagePicker().pickImage(source: ImageSource.gallery);
    if (pickedFile != null) {
      return pickedFile.path;
    }
  }

  return null;
}
