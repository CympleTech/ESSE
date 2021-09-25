import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter/cupertino.dart' show CupertinoActivityIndicator;

class Avatar extends StatelessWidget {
  final double width;
  final String name;
  final Uint8List? avatar;
  final String? avatarPath;
  final bool online;
  final Color onlineColor;
  final bool hasNew;
  final Color hasNewColor;
  final bool loading;
  final bool colorSurface;

  const Avatar(
    {Key? key,
      required this.name,
      this.avatar,
      this.avatarPath,
      this.width = 45.0,
      this.online = false,
      this.onlineColor = Colors.grey,
      this.hasNew = false,
      this.hasNewColor = Colors.red,
      this.loading = false,
      this.colorSurface = true,
  })
  : super(key: key);

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    var showAvatar;
    if (this.avatarPath != null) {
      if (FileSystemEntity.typeSync(this.avatarPath!) !=
        FileSystemEntityType.notFound) {
        showAvatar = FileImage(File(this.avatarPath!));
      }
    } else if (this.avatar != null) {
      showAvatar = MemoryImage(this.avatar!);
    }

    return Container(
      width: this.width,
      height: this.width,
      decoration: showAvatar != null
      ? BoxDecoration(
        image: DecorationImage(image: showAvatar, fit: BoxFit.cover),
        borderRadius: BorderRadius.circular(10.0)
      )
      : BoxDecoration(
        color: this.colorSurface ? color.surface : color.background,
        borderRadius: BorderRadius.circular(10.0)
      ),
      child: Stack(
        alignment: Alignment.center,
        children: <Widget>[
          if (showAvatar == null)
          Text(this.name.length > 0 ? this.name[0].toUpperCase() : "A"),
          if (this.hasNew)
          Positioned(top: 0.0, right: 0.0,
            child: Container(width: 8.0, height: 8.0,
              decoration: BoxDecoration(color: this.hasNewColor, shape: BoxShape.circle),
            ),
          ),
          if (this.online)
          Positioned(bottom: 0.0, right: 0.0,
            child: Container(
              padding: const EdgeInsets.all(2.0),
              decoration: ShapeDecoration(color: color.background, shape: CircleBorder()),
              child: this.loading
              ? CupertinoActivityIndicator(radius: 5.0, animating: true)
              : Container(width: 6.0, height: 6.0,
                decoration: BoxDecoration(color: this.onlineColor, shape: BoxShape.circle),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
