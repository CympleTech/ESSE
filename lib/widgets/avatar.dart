import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/material.dart';

class Avatar extends StatelessWidget {
  final double width;
  final String name;
  final Uint8List avatar;
  final String avatarPath;
  final bool online;
  final bool needOnline;
  final bool hasNew;

  const Avatar(
    {Key key,
      this.width = 45.0,
      this.name,
      this.avatar,
      this.avatarPath,
      this.online = false,
      this.needOnline = true,
      this.hasNew = false})
  : super(key: key);

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    var showAvatar;
    if (this.avatarPath != null) {
      if (FileSystemEntity.typeSync(this.avatarPath) !=
        FileSystemEntityType.notFound) {
        showAvatar = FileImage(File(this.avatarPath));
      }
    } else if (this.avatar != null) {
      showAvatar = MemoryImage(this.avatar);
    }

    return Container(
      width: width,
      height: width,
      decoration: showAvatar != null
      ? BoxDecoration(
        image: DecorationImage(
          image: showAvatar,
          fit: BoxFit.cover,
        ),
        borderRadius: BorderRadius.circular(15.0))
      : BoxDecoration(
        color: color.surface, borderRadius: BorderRadius.circular(15.0)),
      child: Stack(
        alignment: Alignment.center,
        children: <Widget>[
          if (showAvatar == null)
          Text(this.name.length > 0 ? this.name[0].toUpperCase() : "A"),
          if (this.hasNew)
          Positioned(
            top: 0.0,
            right: 0.0,
            child: Container(
              width: 9.0,
              height: 9.0,
              decoration: const BoxDecoration(
                color: Colors.red,
                shape: BoxShape.circle,
              ),
            ),
          ),
          if (this.needOnline)
          Positioned(
            bottom: 0.0,
            right: 0.0,
            child: Container(
              padding: const EdgeInsets.all(2.0),
              decoration: const ShapeDecoration(
                color: Colors.white,
                shape: CircleBorder(),
              ),
              child: Container(
                width: 9.0,
                height: 9.0,
                decoration: BoxDecoration(
                  color: online ? const Color(0xFF0EE50A) : const Color(0xFFEDEDED),
                  shape: BoxShape.circle,
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
