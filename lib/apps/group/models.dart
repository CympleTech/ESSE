import 'package:flutter/material.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/utils/relative_time.dart';
import 'package:esse/widgets/avatar.dart';
import 'package:esse/global.dart';

import 'package:esse/apps/primitives.dart';

class GroupChat {
  int id = 0;
  String gid = '';
  String addr = '';
  String name = '';
  bool isClosed = false;
  bool isLocal = true;

  GroupChat();

  GroupChat.fromList(List params):
    this.id = params[0],
    this.gid = params[1],
    this.addr = params[2],
    this.name = params[3],
    this.isClosed = params[4],
    this.isLocal = params[4];

  Avatar showAvatar({double width = 45.0}) {
    final avatar = Global.avatarPath + this.gid + '.png';
    return Avatar(width: width, name: this.name, avatarPath: avatar);
  }
}

class Member {
  int id;
  int fid;
  String mid;
  String addr;
  String name;
  bool leave;
  bool online = false;

  Member.fromList(List params):
    this.id = params[0],
    this.fid = params[1],
    this.mid = params[2],
    this.addr = params[3],
    this.name = params[4],
    this.leave = params[5];

  Avatar showAvatar({double width = 45.0, bool isOnline = true}) {
    final avatar = Global.avatarPath + this.mid + '.png';
    return Avatar(
      width: width,
      name: this.name,
      avatarPath: avatar,
      online: isOnline && this.online,
      onlineColor: Color(0xFF0EE50A),
    );
  }
}

class Message extends BaseMessage {
  int height = 0;
  int fid = 0;
  int mid = 0;

  Message(int fid, MessageType type, String content) {
    this.fid = fid;
    this.type = type;
    this.content = content;
  }

  Message.fromList(List params) {
    this.id = params[0];
    this.height = params[1];
    this.fid = params[2];
    this.mid = params[3];
    this.isMe = params[4];
    this.type = MessageTypeExtension.fromInt(params[5]);
    this.content = params[6];
    this.isDelivery = params[7];
    this.time = RelativeTime.fromInt(params[8]);
  }
}
