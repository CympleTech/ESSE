import 'package:flutter/material.dart';

import 'package:esse/utils/relative_time.dart';
import 'package:esse/l10n/localizations.dart';

const List<RootDirectory> ROOT_DIRECTORY = [
  RootDirectory.Star,
  RootDirectory.Document,
  RootDirectory.Image,
  RootDirectory.Music,
  RootDirectory.Video,
  RootDirectory.Session,
  RootDirectory.Trash,
];

enum RootDirectory {
  Star,
  Document,
  Image,
  Music,
  Video,
  Session,
  Trash,
}

extension RootDirectoryExtension on RootDirectory {
  List params(AppLocalizations lang) {
    switch (this) {
      case RootDirectory.Star:
        return [Icons.star, lang.star,
          [FilePath.root(RootDirectory.Star, lang.star)]];
      case RootDirectory.Document:
        return [Icons.description, lang.document,
          [FilePath.root(RootDirectory.Document, lang.document)]];
      case RootDirectory.Image:
        return [Icons.image, lang.image,
          [FilePath.root(RootDirectory.Image, lang.image)]];
      case RootDirectory.Music:
        return [Icons.music_note, lang.music,
          [FilePath.root(RootDirectory.Music, lang.music)]];
      case RootDirectory.Video:
        return [Icons.play_circle_filled, lang.video,
          [FilePath.root(RootDirectory.Video, lang.video)]];
      case RootDirectory.Session:
        return [Icons.sms, lang.sessions,
          [FilePath.root(RootDirectory.Session, lang.sessions)]];
      case RootDirectory.Trash:
        return [Icons.auto_delete, lang.trash,
          [FilePath.root(RootDirectory.Trash, lang.trash)]];
    }
  }

  int toInt() {
    switch (this) {
      case RootDirectory.Star:
        return 0;
      case RootDirectory.Trash:
        return 1;
      case RootDirectory.Session:
        return 2;
      case RootDirectory.Document:
        return 3;
      case RootDirectory.Image:
        return 4;
      case RootDirectory.Music:
        return 5;
      case RootDirectory.Video:
        return 6;
    }
  }

  static RootDirectory fromInt(int a) {
    switch (a) {
      case 0:
        return RootDirectory.Star;
      case 1:
        return RootDirectory.Trash;
      case 2:
        return RootDirectory.Session;
      case 3:
        return RootDirectory.Document;
      case 4:
        return RootDirectory.Image;
      case 5:
        return RootDirectory.Music;
      case 6:
        return RootDirectory.Video;
      default:
        return RootDirectory.Trash;
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
  'm4a': FileType.Music,
  'flac': FileType.Music,
  'wav': FileType.Music,
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
        return [Icons.description_rounded, Color(0xFF0b335b)];
      case FileType.Markdown:
        return [Icons.description_rounded, Color(0xFF455A64)];
      case FileType.Other:
        return [Icons.insert_drive_file_rounded, Colors.grey];
    }
  }
}

FileType parseFileType(String name) {
  if (name.endsWith('.quill.json')) {
    return FileType.Post;
  }

  final i = name.lastIndexOf('.');
  if (i > 0) {
    final suffix = name.substring(i + 1);
    if (FILE_TYPES.containsKey(suffix)) {
      return FILE_TYPES[suffix]!;
    }
  }
  return FileType.Other;
}

class FilePath {
  int id = 0;
  String did = '';
  int parent = 0;
  RootDirectory root = RootDirectory.Trash;
  String name = '';
  bool starred = false;
  RelativeTime time = RelativeTime();

  FilePath.root(this.root, this.name);

  static newPostName(String name) {
    return name + '.quill.json';
  }

  static newFolderName(String name) {
    return name + '.dir';
  }

  String directoryName() {
    final i = this.name.lastIndexOf('.');
    if (i < 0) {
      return this.name;
    } else {
      return this.name.substring(0, i);
    }
  }

  String showName() {
    if (isDirectory()) {
      final i = this.name.lastIndexOf('.');
      return this.name.substring(0, i);
    } else if (isPost()){
      final i = this.name.lastIndexOf('.quill');
      return this.name.substring(0, i);
    } else {
      return this.name;
    }
  }

  FileType fileType() {
    return parseFileType(this.name);
  }

  bool isPost() {
    return this.name.endsWith('.quill.json');
  }

  bool isDirectory() {
    return this.name.endsWith('.dir');
  }

  FilePath.fromList(List params) {
    this.id = params[0];
    this.did = params[1];
    this.parent = params[2];
    this.root = RootDirectoryExtension.fromInt(params[3]);
    this.name = params[4];
    this.starred = params[5];
    this.time = RelativeTime.fromInt(params[6]);
  }
}
