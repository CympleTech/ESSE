import 'package:esse/utils/relative_time.dart';
import 'package:esse/widgets/avatar.dart';
import 'package:esse/global.dart';

import 'package:esse/apps/primitives.dart';

class Friend {
  int id;
  String gid;
  String name;
  String addr;
  String remark;
  bool isClosed;
  RelativeTime time;

  // new friend from network
  Friend(this.gid, this.name, this.addr) {
    this.isClosed = false;
  }

  Avatar showAvatar({double width = 45.0}) {
    final avatar = Global.avatarPath + this.gid + '.png';
    return Avatar(width: width, name: this.name, avatarPath: avatar);
  }

  Friend.fromList(List params) {
    this.id = params[0];
    this.gid = params[1];
    this.addr = params[2];
    this.name = params[3];
    this.remark = params[4];
    this.isClosed = params[5];
    this.time = RelativeTime.fromInt(params[6]);
  }
}

class Request {
  int id;
  String gid;
  String addr;
  String name;
  String remark;
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
  String hash;
  int fid;

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
