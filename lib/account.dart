import 'package:flutter/material.dart';

import 'dart:convert';
import 'dart:typed_data';

import 'package:esse/widgets/avatar.dart';

class Account {
  String gid;
  String name;
  String lock;
  Uint8List avatar;
  bool online;
  bool hasNew;

  Account(String gid, String name, [String lock = "", String avatar = "", bool online = false]) {
    this.gid = gid;
    this.name = name;
    this.lock = lock;
    this.updateAvatar(avatar);
    this.online = online;
    this.hasNew = false;
  }

  String get id => 'EH' + this.gid.toUpperCase();

  String encodeAvatar() {
    if (this.avatar != null && this.avatar.length > 1) {
      return base64.encode(this.avatar);
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

  String printShortId() {
    final id = this.id;
    final len = id.length;
    if (len > 4) {
      return id.substring(0, 4) + "..." + id.substring(len - 4, len);
    } else {
      return id;
    }
  }

  String printId() {
    final id = this.id;
    final len = id.length;
    if (len > 8) {
      return id.substring(0, 8) + "..." + id.substring(len - 8, len);
    } else {
      return id;
    }
  }

  Avatar showAvatar({double width = 45.0, bool online = false, bool needOnline = true}) {
    return Avatar(
      width: width,
      name: this.name,
      avatar: this.avatar,
      online: true,
      onlineColor: this.online ? const Color(0xFF0EE50A) : const Color(0xFFEDEDED),
      hasNew: this.hasNew,
    );
  }
}
