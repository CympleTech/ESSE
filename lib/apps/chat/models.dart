import 'package:esse/utils/relative_time.dart';
import 'package:esse/widgets/avatar.dart';
import 'package:esse/global.dart';

class Friend {
  int id;
  String gid;
  String name;
  String addr;
  String remark;
  bool isTop;
  bool isClosed;
  RelativeTime lastMessageTime;
  String lastMessageContent;
  bool lastMessageReaded;
  bool online = false;

  // new friend from network
  Friend(this.gid, this.name, this.addr) {
    this.isTop = false;
    this.isClosed = false;
    this.lastMessageTime = RelativeTime();
    this.lastMessageContent = '';
    this.lastMessageReaded = true;
  }

  Avatar showAvatar({double width = 45.0, bool needOnline = true}) {
    final avatar = Global.avatarPath + this.gid + '.png';
    return Avatar(
      width: width,
      name: this.name,
      avatarPath: avatar,
      online: this.online,
      needOnline: needOnline,
      hasNew: !this.lastMessageReaded,
    );
  }

  updateLastMessage(Message msg, bool isReaded) {
    this.lastMessageTime = msg.time;
    this.lastMessageContent = msg.shortShow();
    this.lastMessageReaded = isReaded;
  }

  static String betterPrint(String info) {
    if (info == null) {
      return '';
    }
    final len = info.length;
    if (len > 8) {
      return info.substring(0, 8) + '...' + info.substring(len - 6, len);
    } else {
      return info;
    }
  }

  Friend.fromList(List params) {
    this.id = params[0];
    this.gid = params[1];
    this.addr = params[2];
    this.name = params[3];
    this.remark = params[4];
    this.isTop = params[5] == "1";
    this.isClosed = params[6] == "1";
    this.lastMessageTime = RelativeTime.fromInt(params[7]);
    this.lastMessageContent = params[8];
    this.lastMessageReaded = params[9];
    this.online = params[10] == "1";
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
    return Avatar(
        width: width, name: this.name, avatarPath: avatar, needOnline: false);
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

enum MessageType {
  String,
  Image,
  File,
  Contact,
  Emoji,
  Record,
  Phone,
  Video,
}

// use 00-99
extension MessageTypeExtension on MessageType {
  int toInt() {
    switch (this) {
      case MessageType.String:
        return 0;
      case MessageType.Image:
        return 1;
      case MessageType.File:
        return 2;
      case MessageType.Contact:
        return 3;
      case MessageType.Emoji:
        return 4;
      case MessageType.Record:
        return 5;
      case MessageType.Phone:
        return 6;
      case MessageType.Video:
        return 7;
      default:
        return 0;
    }
  }

  static MessageType fromInt(int s) {
    switch (s) {
      case 0:
        return MessageType.String;
      case 1:
        return MessageType.Image;
      case 2:
        return MessageType.File;
      case 3:
        return MessageType.Contact;
      case 4:
        return MessageType.Emoji;
      case 5:
        return MessageType.Record;
      case 6:
        return MessageType.Phone;
      case 7:
        return MessageType.Video;
      default:
        return MessageType.String;
    }
  }
}

class Message {
  int id;
  String hash;
  int fid;
  bool isMe = true;
  MessageType type;
  String content;
  bool isDelivery = false;
  RelativeTime time = RelativeTime();

  Message(this.fid, this.type, this.content);

  List showContact() {
    var name = '';
    var did = '';
    var addr = '';

    var i_name = this.content.indexOf(';;');
    if (i_name > 0) {
      name = this.content.substring(0, i_name).replaceAll('-;', ';');
    }
    var raw = this.content.substring(i_name + 2);
    var i_did = raw.indexOf(';;');
    if (i_did > 0) {
      did = raw.substring(0, i_did);
    }
    addr = raw.substring(i_did + 2);

    return [name, did, addr, Global.avatarPath + did + '.png'];
  }

  static String rawRecordName(int time, String name) {
    return time.toString() + '-' + name;
  }

  List showRecordTime() {
    final len = this.content.indexOf('-');
    if (len > 0) {
      final time = int.parse(this.content.substring(0, len));
      final path = this.content.substring(len + 1);
      return [time, path];
    } else {
      return [0, this.content];
    }
  }

  String shortShow() {
    switch (this.type) {
      case MessageType.Image:
        return '[IMAGE]';
      case MessageType.Record:
        return '[RECORD]';
      case MessageType.Phone:
        return '[PHONE]';
      case MessageType.Video:
        return '[VIDEO]';
      case MessageType.Contact:
        return '[CONTACT CARD]';
      default:
        return this.content;
    }
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
