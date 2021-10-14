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
    } else {
      return this.path.last;
    }
  }

  bool isDirectory() {
    return this.path.last.endsWith('.dir');
  }
}
