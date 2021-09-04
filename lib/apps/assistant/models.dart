import 'package:esse/utils/relative_time.dart';
import 'package:esse/global.dart';

enum MessageType {
  String,
  Image,
  File,
  Contact,
  Emoji,
  Record,
  Answer,
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
      case MessageType.Answer:
        return 6;
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
        return MessageType.Answer;
      default:
        return MessageType.String;
    }
  }
}

class Message {
  int id = 0;
  MessageType qType = MessageType.String;
  String qContent = '';
  MessageType aType = MessageType.String;
  String aContent = '';
  RelativeTime time = RelativeTime();

  Message(this.qType, this.qContent);

  static List showContact(String content) {
    var name = '';
    var did = '';
    var addr = '';

    var iName = content.indexOf(';;');
    if (iName > 0) {
      name = content.substring(0, iName).replaceAll('-;', ';');
    }
    var raw = content.substring(iName + 2);
    var iDid = raw.indexOf(';;');
    if (iDid > 0) {
      did = raw.substring(0, iDid);
    }
    addr = raw.substring(iDid + 2);

    return [name, did, addr, Global.avatarPath + did + '.png'];
  }

  static String rawRecordName(int time, String name) {
    return time.toString() + '-' + name;
  }

  static List showRecordTime(String content) {
    final len = content.indexOf('-');
    if (len > 0) {
      final time = int.parse(content.substring(0, len));
      final path = content.substring(len + 1);
      return [time, path];
    } else {
      return [0, content];
    }
  }

  Message.fromList(List params):
    this.id = params[0],
    this.qType = MessageTypeExtension.fromInt(params[1]),
    this.qContent = params[2],
    this.aType = MessageTypeExtension.fromInt(params[3]),
    this.aContent = params[4],
    this.time = RelativeTime.fromInt(params[5]);

  update(List params) {
    // params[0] is id.
    this.aType = MessageTypeExtension.fromInt(params[1]);
    this.aContent = params[2];
  }
}
