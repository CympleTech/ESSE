import 'package:flutter/material.dart';

// AssetImage('assets/images/file_default.png'),
// AssetImage('assets/images/file_image.png'),
// AssetImage('assets/images/file_pdf.png'),
// AssetImage('assets/images/file_word.png'),
// AssetImage('assets/images/file_sheet.png'),
// AssetImage('assets/images/file_markdown.png'),
// AssetImage('assets/images/file_video.png'),
// AssetImage('assets/images/dir_folder.svg'),

const List FILE_IMAGES = [
  [Icons.insert_drive_file_rounded, Colors.grey], // default file
  [Icons.image_rounded, Colors.green], // image
  [Icons.chrome_reader_mode_rounded, Color(0xFFFF5722)], // pdf
  [Icons.text_snippet_rounded, Color(0xFF1976d2)], // word
  [Icons.table_chart_rounded, Color(0xFF4CAF50)], // sheet
  [Icons.slideshow_rounded, Color(0xFFFF6D00)], // slide
  [Icons.article_rounded, Color(0xFF455A64)], // markdown
  [Icons.play_circle_fill_rounded, Colors.red], // video
  [Icons.folder_rounded, Colors.blue], // folder
];

const Map<String, int> FILE_TYPES = {
  'jpg': 1,
  'jpeg': 1,
  'png': 1,
  'svg': 1,
  'pdf': 2,
  'doc': 3,
  'docx': 3,
  'xls': 4,
  'xlsx': 4,
  'ppt': 5,
  'pptx': 5,
  'md': 6,
  'mp4': 7,
  'dir': 8,
};

Widget fileIcon(String name, double size) {
  final i = name.lastIndexOf('.');
  if (i > 0) {
    final suffix = name.substring(i + 1);
    if (FILE_TYPES.containsKey(suffix)) {
      final index = FILE_TYPES[suffix];
      return Icon(FILE_IMAGES[index][0], color: FILE_IMAGES[index][1], size: size);
    }
  }

  return Icon(FILE_IMAGES[0][0], color: FILE_IMAGES[0][1], size: size);
}
