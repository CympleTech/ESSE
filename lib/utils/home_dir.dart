import 'dart:io' show Directory;

import 'package:path_provider/path_provider.dart';

Future<String> homeDir() async {
  final directory = await getApplicationDocumentsDirectory();
  //final directory = await getExternalStorageDirectory();
  final myDir = new Directory(directory.path + '/esse-pre');
  final isThere = await myDir.exists();
  if (!isThere) {
    await myDir.create(recursive: true);
  }
  return myDir.path;
}
