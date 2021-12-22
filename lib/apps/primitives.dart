import 'package:flutter/material.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/utils/relative_time.dart';
import 'package:esse/global.dart';

enum GroupType {
  Tmp,
  Open,
  Private,
  Encrypted,
}

extension GroupTypeExtension on GroupType {
  int toInt() {
    switch (this) {
      case GroupType.Tmp:
        return 0;
      case GroupType.Open:
        return 1;
      case GroupType.Private:
        return 2;
      case GroupType.Encrypted:
        return 3;
      default:
        return 0;
    }
  }

  String lang(AppLocalizations lang) {
    switch (this) {
      case GroupType.Encrypted:
        return lang.groupTypeEncrypted;
      case GroupType.Private:
        return lang.groupTypePrivate;
      case GroupType.Open:
        return lang.groupTypeOpen;
      case GroupType.Tmp:
        return lang.groupChat;
    }
  }

  static GroupType fromInt(int s) {
    switch (s) {
      case 0:
        return GroupType.Tmp;
      case 1:
        return GroupType.Open;
      case 2:
        return GroupType.Private;
      case 3:
        return GroupType.Encrypted;
      default:
        return GroupType.Tmp;
    }
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
  int id = 0;
  bool isMe = true;
  MessageType type = MessageType.String;
  String content = '';
  bool? isDelivery;
  RelativeTime time = RelativeTime();

  List showContact() {
    var name = '';
    var did = '';
    var addr = '';

    var iName = this.content.indexOf(';;');
    if (iName > 0) {
      name = this.content.substring(0, iName).replaceAll('-;', ';');
    }
    var raw = this.content.substring(iName + 2);
    var iDid = raw.indexOf(';;');
    if (iDid > 0) {
      did = raw.substring(0, iDid);
    }
    addr = raw.substring(iDid + 2);

    return [name, did, addr, Global.avatarPath + did + '.png'];
  }

  List showInvite() {
    var type = GroupType.Tmp;
    var gid = '';
    var addr = '';
    var name = '';
    var proof = '';
    var key = '';

    final iType = this.content.indexOf(';;');
    if (iType > 0) {
      type = GroupTypeExtension.fromInt(int.parse(this.content.substring(0, iType)));
    }

    final raw_0 = this.content.substring(iType + 2);
    final iGid = raw_0.indexOf(';;');
    if (iGid > 0) {
      gid = raw_0.substring(0, iGid);
    }

    final raw_1 = raw_0.substring(iGid + 2);
    final iAddr = raw_1.indexOf(';;');
    if (iAddr > 0) {
      addr = raw_1.substring(0, iAddr);
    }

    final raw_2 = raw_1.substring(iAddr + 2);
    final iName = raw_2.indexOf(';;');
    if (iName > 0) {
      name = raw_2.substring(0, iName).replaceAll('-;', ';');
    } else {
      name = raw_2.replaceAll('-;', ';');
      return [type, gid, addr, name, proof, key];
    }

    final raw_3 = raw_2.substring(iName + 2);
    final iProof = raw_3.indexOf(';;');
    if (iProof > 0) {
      proof = raw_3.substring(0, iProof);
      key = raw_3.substring(iProof + 2);
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

PopupMenuEntry<int> menuItem(Color color, int value, IconData icon, String text) {
  return PopupMenuItem<int>(
    value: value,
    child: Row(
      children: [
        Icon(icon, color: color),
        Padding(
          padding: const EdgeInsets.only(left: 20.0, right: 10.0),
          child: Text(text, style: TextStyle(color: Colors.black, fontSize: 16.0)),
        )
      ]
    ),
  );
}
