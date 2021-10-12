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
        return [Icons.star, lang.star];
      case RootDirectory.Document:
        return [Icons.description, lang.document];
      case RootDirectory.Image:
        return [Icons.image, lang.image];
      case RootDirectory.Music:
        return [Icons.music_note, lang.music];
      case RootDirectory.Video:
        return [Icons.play_circle_filled, lang.video];
      case RootDirectory.Trash:
        return [Icons.auto_delete, lang.trash];
    }
  }
}
