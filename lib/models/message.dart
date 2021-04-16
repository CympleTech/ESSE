import 'package:esse/utils/relative_time.dart';
import 'package:esse/global.dart';

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
