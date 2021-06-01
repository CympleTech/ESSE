import 'package:esse/utils/relative_time.dart';
import 'package:esse/widgets/avatar.dart';
import 'package:esse/global.dart';
import 'package:esse/apps/group_chat/models.dart' show GroupType, GroupTypeExtension;

enum MessageType {
  String,
  Image,
  File,
  Contact,
  Emoji,
  Record,
  Phone,
  Video,
  Invite,
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
      case MessageType.Invite:
        return 8;
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
      case 8:
        return MessageType.Invite;
      default:
        return MessageType.String;
    }
  }
}

class BaseMessage {
  int id;
  bool isMe = true;
  MessageType type;
  String content;
  bool isDelivery = false;
  RelativeTime time = RelativeTime();

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

  List showInvite() {
    var type = GroupType.Open;
    var gid = '';
    var addr = '';
    var name = '';
    var proof = '';
    var key = '';

    final i_type = this.content.indexOf(';;');
    if (i_type > 0) {
      type = GroupTypeExtension.fromInt(int.parse(this.content.substring(0, i_type)));
    }

    final raw_0 = this.content.substring(i_type + 2);
    final i_gid = raw_0.indexOf(';;');
    if (i_gid > 0) {
      gid = raw_0.substring(0, i_gid);
    }

    final raw_1 = raw_0.substring(i_gid + 2);
    final i_addr = raw_1.indexOf(';;');
    if (i_addr > 0) {
      addr = raw_1.substring(0, i_addr);
    }

    final raw_2 = raw_1.substring(i_addr + 2);
    final i_name = raw_2.indexOf(';;');
    if (i_name > 0) {
      name = raw_2.substring(0, i_name).replaceAll('-;', ';');
    }

    final raw_3 = raw_2.substring(i_name + 2);
    final i_proof = raw_3.indexOf(';;');
    if (i_proof > 0) {
      proof = raw_3.substring(0, i_proof);
      key = raw_3.substring(i_proof + 2);
    } else {
      proof = raw_3;
    }

    return [type, gid, addr, name, proof, key];
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
}
