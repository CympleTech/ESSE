import 'package:flutter/material.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/utils/relative_time.dart';
import 'package:esse/widgets/avatar.dart';
import 'package:esse/global.dart';

import 'package:esse/apps/primitives.dart';

enum GroupType {
  Encrypted,
  Common,
  Open,
}

enum CheckType {
  Allow,
  None,
  Suspend,
  Deny,
  Wait,
}

extension GroupTypeExtension on GroupType {
  int toInt() {
    switch (this) {
      case GroupType.Encrypted:
        return 0;
      case GroupType.Common:
        return 1;
      case GroupType.Encrypted:
        return 2;
      default:
        return 0;
    }
  }

  static GroupType fromInt(int s) {
    switch (s) {
      case 0:
        return GroupType.Encrypted;
      case 1:
        return GroupType.Common;
      case 2:
        return GroupType.Open;
      default:
        return GroupType.Encrypted;
    }
  }
}

extension CheckTypeExtension on CheckType {
  List lang(AppLocalizations lang) {
    switch (this) {
      case CheckType.Allow:
        return [lang.groupCheckTypeAllow, true];
      case CheckType.None:
        return [lang.groupCheckTypeNone, false];
      case CheckType.Suspend:
        return [lang.groupCheckTypeSuspend, false];
      case CheckType.Deny:
        return [lang.groupCheckTypeDeny, false];
      default:
        return ['', false];
    }
  }

  static CheckType fromInt(int s) {
    switch (s) {
      case 0:
        return CheckType.Allow;
      case 1:
        return CheckType.None;
      case 2:
        return CheckType.Suspend;
      case 3:
        return CheckType.Deny;
      default:
        return CheckType.Deny;
    }
  }
}

class GroupChat {
  int id;
  String owner;
  String gid;
  GroupType type;
  String addr;
  String name;
  String bio;
  bool isOk;
  bool isClosed;
  bool isNeedAgree;

  GroupChat.fromList(List params) {
    this.id = params[0];
    this.owner = params[1];
    this.gid = params[2];
    this.type = GroupTypeExtension.fromInt(params[3]);
    this.addr = params[4];
    this.name = params[5];
    this.bio = params[6];
    this.isOk = params[7];
    this.isClosed = params[8];
    this.isNeedAgree = params[9];
  }

  Avatar showAvatar({double width = 45.0}) {
    final avatar = Global.avatarPath + this.gid + '.png';
    return Avatar(width: width, name: this.name, avatarPath: avatar);
  }
}

class Request {
  int id;
  int fid;
  String gid;
  String addr;
  String name;
  String remark;
  bool ok;
  bool over;
  RelativeTime time;

  bool get isMe => this.fid == 0;

  overIt(bool ok) {
    this.ok = ok;
    this.over = true;
  }

  Avatar showAvatar([double width = 45.0]) {
    final avatar = Global.avatarPath + this.gid + '.png';
    return Avatar(width: width, name: this.name, avatarPath: avatar);
  }

  Request.fromList(List params) {
    this.id = params[0];
    this.fid = params[1];
    this.gid = params[2];
    this.addr = params[3];
    this.name = params[4];
    this.remark = params[5];
    this.ok = params[6];
    this.over = params[7];
    this.time = RelativeTime.fromInt(params[8]);
  }
}

class Member {
  int id;
  int fid;
  String mid;
  String addr;
  String name;
  bool isManager;
  bool isBlock;
  bool online = false;

  Member.fromList(List params) {
    this.id = params[0];
    this.fid = params[1];
    this.mid = params[2];
    this.addr = params[3];
    this.name = params[4];
    this.isManager = params[5];
    this.isBlock = params[6];
  }

  Avatar showAvatar({double width = 45.0, colorSurface = true}) {
    final avatar = Global.avatarPath + this.mid + '.png';
    return Avatar(
      width: width,
      name: this.name,
      avatarPath: avatar,
      online: this.online,
      onlineColor: Color(0xFF0EE50A),
      colorSurface: colorSurface,
    );
  }
}

class Message extends BaseMessage {
  int height;
  int fid;
  int mid;

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
