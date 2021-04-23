import 'package:esse/utils/relative_time.dart';
import 'package:esse/global.dart';

enum MessageType {
  String,
  Image,
  File,
  Contact,
  Emoji,
  Record,
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
      default:
        return MessageType.String;
    }
  }
}

class Message {
  int id;
  MessageType q_type;
  String q_content;
  MessageType a_type;
  String a_content;
  RelativeTime time = RelativeTime();

  Message(this.q_type, this.q_content);

  static List showContact(String content) {
    var name = '';
    var did = '';
    var addr = '';

    var i_name = content.indexOf(';;');
    if (i_name > 0) {
      name = content.substring(0, i_name).replaceAll('-;', ';');
    }
    var raw = content.substring(i_name + 2);
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

  String shortShow() {
    switch (this.q_type) {
      case MessageType.Image:
        return '[IMAGE]';
      case MessageType.Record:
        return '[RECORD]';
      case MessageType.Contact:
        return '[CONTACT CARD]';
      default:
        return this.q_content;
    }
  }

  Message.fromList(List params) {
    this.id = params[0];
    this.q_type = MessageTypeExtension.fromInt(params[1]);
    this.q_content = params[2];
    this.a_type = MessageTypeExtension.fromInt(params[3]);
    this.a_content = params[4];
    this.time = RelativeTime.fromInt(params[5]);
  }

  update(List params) {
    // params[0] is id.
    this.a_type = MessageTypeExtension.fromInt(params[1]);
    this.a_content = params[2];
  }
}
