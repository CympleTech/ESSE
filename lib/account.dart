import 'package:flutter/material.dart';

import 'dart:convert';
import 'dart:typed_data';

import 'package:esse/widgets/avatar.dart';

class Account {
  String gid = '';
  String name = '';
  String lock = '';
  Uint8List? avatar;
  bool online = false;
  bool hasNew = false;

  Account(String gid, String name, [String lock = "", String avatar = "", bool online = false]) {
    this.gid = gid;
    this.name = name;
    this.lock = lock;
    this.updateAvatar(avatar);
    this.online = online;
    this.hasNew = false;
  }

  String encodeAvatar() {
    if (this.avatar != null && this.avatar!.length > 1) {
      return base64.encode(this.avatar!);
    } else {
      return '';
    }
  }

  void updateAvatar(String avatar) {
    if (avatar.length > 1) {
      this.avatar = base64.decode(avatar);
    } else {
      this.avatar = null;
    }
  }

  Avatar showAvatar({double width = 45.0, bool online = false, bool needOnline = true}) {
    return Avatar(
      width: width,
      name: this.name,
      avatar: this.avatar,
      online: needOnline,
      onlineColor: this.online ? const Color(0xFF0EE50A) : const Color(0xFFEDEDED),
      hasNew: this.hasNew,
    );
  }
}
