import 'package:flutter/material.dart';

import 'package:esse/utils/relative_time.dart';
import 'package:esse/widgets/avatar.dart';
import 'package:esse/global.dart';

import 'package:esse/apps/primitives.dart';

class Friend {
  int id = 0;
  String gid = '';
  String name = '';
  String addr = '';
  String wallet = '';
  String remark = '';
  bool isClosed = false;
  RelativeTime time = RelativeTime();
  bool online = false;

  // new friend from network
  Friend(this.gid, this.name, this.addr);

  Avatar showAvatar({bool needOnline = false, double width = 45.0}) {
    final avatar = Global.avatarPath + this.gid + '.png';
    if (needOnline) {
      return Avatar(width: width, name: this.name, avatarPath: avatar,
        online: this.online,
        onlineColor: Color(0xFF0EE50A),
      );
    } else {
      return Avatar(width: width, name: this.name, avatarPath: avatar);
    }
  }

  Friend.fromList(List params) {
    this.id = params[0];
    this.gid = params[1];
    this.addr = params[2];
    this.name = params[3];
    this.wallet = params[4];
    this.remark = params[5];
    this.isClosed = params[6];
    this.time = RelativeTime.fromInt(params[7]);
    if (params.length == 9) {
      this.online = params[8];
    }
  }
}

class Request {
  int id = 0;
  String gid = '';
  String addr = '';
  String name = '';
  String remark = '';
  bool isMe = true;
  bool ok = false;
  bool over = false;
  bool isDelivery = false;
  RelativeTime time = RelativeTime();

  Request(this.gid, this.addr, this.name, this.remark);

  overIt(bool isOk) {
    this.over = true;
    this.ok = isOk;
  }

  Friend toFriend(String gid) {
    return Friend(gid, this.name, this.addr);
  }

  Avatar showAvatar([double width = 45.0]) {
    final avatar = Global.avatarPath + this.gid + '.png';
    return Avatar(width: width, name: this.name, avatarPath: avatar);
  }

  Request.fromList(List params) {
    this.id = params[0];
    this.gid = params[1];
    this.addr = params[2];
    this.name = params[3];
    this.remark = params[4];
    this.isMe = params[5];
    this.ok = params[6];
    this.over = params[7];
    this.isDelivery = params[8];
    this.time = RelativeTime.fromInt(params[9]);
  }
}

class Message extends BaseMessage {
  String hash = '';
  int fid = 0;

  Message(int fid, MessageType type, String content) {
    this.fid = fid;
    this.type = type;
    this.content = content;
  }

  Message.fromList(List params) {
    this.id = params[0];
    this.hash = params[1];
    this.fid = params[2];
    this.isMe = params[3];
    this.type = MessageTypeExtension.fromInt(params[4]);
    this.content = params[5];
    this.isDelivery = params[6];
    this.time = RelativeTime.fromInt(params[7]);
  }
}
