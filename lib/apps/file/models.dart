import 'package:flutter/material.dart';
import 'package:esse/l10n/localizations.dart';

const List<RootDirectory> ROOT_DIRECTORY = [
  RootDirectory.Star,
  RootDirectory.Document,
  RootDirectory.Image,
  RootDirectory.Music,
  RootDirectory.Video,
  RootDirectory.Trash,
];

enum RootDirectory {
  Star,
  Document,
  Image,
  Music,
  Video,
  Trash,
}

extension InnerServiceExtension on RootDirectory {
  List params(AppLocalizations lang) {
    switch (this) {
      case RootDirectory.Star:
        return [Icons.star, lang.star, FilePath.root(RootDirectory.Star)];
      case RootDirectory.Document:
        return [Icons.description, lang.document, FilePath.root(RootDirectory.Document)];
      case RootDirectory.Image:
        return [Icons.image, lang.image, FilePath.root(RootDirectory.Image)];
      case RootDirectory.Music:
        return [Icons.music_note, lang.music, FilePath.root(RootDirectory.Music)];
      case RootDirectory.Video:
        return [Icons.play_circle_filled, lang.video, FilePath.root(RootDirectory.Video)];
      case RootDirectory.Trash:
        return [Icons.auto_delete, lang.trash, FilePath.root(RootDirectory.Trash)];
    }
  }
}

const Map<String, FileType> FILE_TYPES = {
  'dir': FileType.Folder,
  'quill.json': FileType.Post,
  'jpg': FileType.Image,
  'jpeg': FileType.Image,
  'png': FileType.Image,
  'svg': FileType.Image,
  'pdf': FileType.Pdf,
  'doc': FileType.Word,
  'docx': FileType.Word,
  'xls': FileType.Sheet,
  'xlsx': FileType.Sheet,
  'ppt': FileType.Slide,
  'pptx': FileType.Slide,
  'md': FileType.Markdown,
  'mp4': FileType.Video,
  'mp3': FileType.Music,
};

enum FileType {
  Folder,
  Post,
  Image,
  Music,
  Video,
  Pdf,
  Slide,
  Sheet,
  Word,
  Markdown,
  Other,
}

// AssetImage('assets/images/file_default.png'),
// AssetImage('assets/images/file_image.png'),
// AssetImage('assets/images/file_pdf.png'),
// AssetImage('assets/images/file_word.png'),
// AssetImage('assets/images/file_sheet.png'),
// AssetImage('assets/images/file_markdown.png'),
// AssetImage('assets/images/file_video.png'),
// AssetImage('assets/images/dir_folder.svg'),

extension FileTypeExtension on FileType {
  List params() {
    switch (this) {
      case FileType.Folder:
        return [Icons.folder_rounded, Colors.blue];
      case FileType.Post:
        return [Icons.article_rounded, Color(0xFF6174FF)];
      case FileType.Image:
        return [Icons.image_rounded, Colors.green];
      case FileType.Music:
        return [Icons.music_note_rounded, Colors.red];
      case FileType.Video:
        return [Icons.play_circle_fill_rounded, Colors.red];
      case FileType.Pdf:
        return [Icons.chrome_reader_mode_rounded, Color(0xFFFF5722)];
      case FileType.Slide:
        return [Icons.slideshow_rounded, Color(0xFFFF6D00)];
      case FileType.Sheet:
        return [Icons.table_chart_rounded, Color(0xFF4CAF50)];
      case FileType.Word:
        return [Icons.description_rounded, Color(0xFF1976d2)];
      case FileType.Markdown:
        return [Icons.description_rounded, Color(0xFF455A64)];
      case FileType.Other:
        return [Icons.insert_drive_file_rounded, Colors.grey];
    }
  }
}

class FilePath {
  RootDirectory root;
  List<String> path = [];
  String get fullName => this.path.last;

  FilePath.root(this.root);

  FilePath(this.root, this.path);

  static FilePath next(FilePath file, String name) {
    final root = file.root;
    List<String> path = List.from(file.path);
    path.add(name);
    return FilePath(root, path);
  }

  static FilePath prev(FilePath file) {
    final root = file.root;
    List<String> path = List.from(file.path);
    if (path.length == 0) {
      return FilePath.root(root);
    } else {
      path.removeLast();
      return FilePath(root, path);
    }
  }

  static directoryName(String name) {
    final i = name.lastIndexOf('.');
    return name.substring(0, i);
  }

  void add(String next) {
    this.path.add(next);
  }

  String name() {
    if (isDirectory()) {
      final i = this.path.last.lastIndexOf('.');
      return this.path.last.substring(0, i);
    } else if (isPost()){
      final i = this.path.last.lastIndexOf('.quill');
      return this.path.last.substring(0, i);
    } else {
      return this.path.last;
    }
  }

  FileType fileType() {
    if (isDirectory()) {
      return FileType.Folder;
    }

    if (isPost()) {
      return FileType.Post;
    }

    final i = this.path.last.lastIndexOf('.');
    if (i > 0) {
      final suffix = this.path.last.substring(i + 1);
      if (FILE_TYPES.containsKey(suffix)) {
        return FILE_TYPES[suffix]!;
      }
    }
    return FileType.Other;
  }

  bool isPost() {
    return this.path.last.endsWith('.quill.json');
  }

  bool isDirectory() {
    return this.path.last.endsWith('.dir');
  }
}
