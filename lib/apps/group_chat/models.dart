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
  bool isTop;
  bool isOk;
  bool isClosed;
  bool isNeedAgree;
  RelativeTime lastTime;
  String lastContent;
  bool lastReaded;
  bool online = false;

  GroupChat.fromList(List params) {
    this.id = params[0];
    this.owner = params[1];
    this.gid = params[2];
    this.type = GroupTypeExtension.fromInt(params[3]);
    this.addr = params[4];
    this.name = params[5];
    this.bio = params[6];
    this.isTop = params[7] == "1";
    this.isOk = params[8] == "1";
    this.isClosed = params[9] == "1";
    this.isNeedAgree = params[10] == "1";
    this.lastTime = RelativeTime.fromInt(params[11]);
    this.lastContent = params[12];
    this.lastReaded = params[13] == "1";
    this.online = params[14] == "1";
  }

  Avatar showAvatar({double width = 45.0, bool needOnline = true}) {
    final avatar = Global.avatarPath + this.gid + '.png';
    return Avatar(
      width: width,
      name: this.name,
      avatarPath: avatar,
      online: this.online,
      needOnline: needOnline,
      hasNew: !this.lastReaded,
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
    this.isMe = params[2];
    this.fid = params[3];
    this.mid = params[4];
    this.type = MessageTypeExtension.fromInt(params[5]);
    this.content = params[6];
    this.isDelivery = params[7];
    this.time = RelativeTime.fromInt(params[8]);
  }
}
