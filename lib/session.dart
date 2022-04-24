import 'package:flutter/material.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/utils/relative_time.dart';
import 'package:esse/widgets/avatar.dart';
import 'package:esse/global.dart';
import 'package:esse/apps/service/models.dart';
import 'package:esse/apps/primitives.dart';

enum SessionType {
  Chat,
  Group,
  Device,
  Jarvis,
}

extension SessionTypeExtension on SessionType {
  static SessionType fromInt(int s) {
    switch (s) {
      case 0:
        return SessionType.Chat;
      case 1:
        return SessionType.Group;
      case 2:
        return SessionType.Device;
      case 3:
        return SessionType.Jarvis;
      default:
        return SessionType.Chat;
    }
  }
}

enum OnlineType {
  Waiting,
  Active,
  Suspend,
  Lost,
}

class Session {
  int id;
  int fid;
  String pid;
  String addr;
  SessionType type;
  String name;
  bool isTop;
  bool isClose;
  RelativeTime lastTime;
  String lastContent;
  bool lastReaded;
  OnlineType online;

  static List innerService(InnerService service, AppLocalizations lang) {
    final params = service.params(lang);
    final avatar = Container(
      padding: const EdgeInsets.all(6.0),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(10.0),
      ),
      child: Image.asset(params[2]),
    );
    final name = params[0];
    final bio = params[1];

    return [avatar, name, bio];
  }

  String onlineLang(AppLocalizations lang) {
    switch (this.online) {
      case OnlineType.Waiting:
        return lang.onlineWaiting;
      case OnlineType.Active:
        return lang.onlineActive;
      case OnlineType.Suspend:
        return lang.onlineSuspend;
      case OnlineType.Lost:
        return lang.onlineLost;
    }
  }

  bool isActive() {
    return this.online == OnlineType.Active ||
        this.online == OnlineType.Suspend;
  }

  String content(AppLocalizations lang) {
    MessageType type = MessageType.String;
    String raw = this.lastContent;

    final msgType = this.lastContent.indexOf(':');
    if (msgType > 0) {
      type = MessageTypeExtension.fromInt(
          int.parse(this.lastContent.substring(0, msgType)));
      raw = this.lastContent.substring(msgType + 1);
    }

    switch (type) {
      case MessageType.String:
        return raw;
      case MessageType.Image:
        return "[${lang.album}]";
      case MessageType.File:
        return "[${lang.file}]";
      case MessageType.Contact:
        return "[${lang.contactCard}]";
      case MessageType.Emoji:
        return "[${lang.emoji}]";
      case MessageType.Record:
        return "[${lang.record}]";
      case MessageType.Phone:
        return "[${lang.others}]";
      case MessageType.Video:
        return "[${lang.others}]";
      case MessageType.Invite:
        return "[${lang.invite}]";
      case MessageType.Transfer:
        return "[${lang.transfer}]";
    }
  }

  List parse(AppLocalizations lang) {
    switch (this.type) {
      case SessionType.Chat:
        return [
          showAvatar(),
          this.name,
          content(lang),
          this.lastTime.toString(),
          null
        ];
      case SessionType.Group:
        return [
          showAvatar(),
          this.name,
          content(lang),
          this.lastTime.toString(),
          Icons.groups
        ];
      case SessionType.Jarvis:
        final params = Session.innerService(InnerService.Jarvis, lang);
        return [params[0], params[1], params[2], '', Icons.campaign];
      default:
        return [];
    }
  }

  Avatar showAvatar({double width = 45.0}) {
    String avatar = Global.avatarPath;
    switch (this.type) {
      case SessionType.Chat:
        avatar = avatar + this.pid + '.png';
        break;
      case SessionType.Group:
        avatar = avatar + 'group_' + this.pid + '.png';
        break;
      default:
        break;
    }

    Color color;
    switch (this.online) {
      case OnlineType.Active:
        color = Color(0xFF0EE50A);
        break;
      case OnlineType.Suspend:
        color = Colors.blue;
        break;
      default:
        color = Color(0xFFADB0BB);
        break;
    }

    if (this.isClose) {
      color = Color(0xFFADB0BB);
    }

    return Avatar(
      width: width,
      name: this.name,
      avatarPath: avatar,
      online: this.online != OnlineType.Lost,
      onlineColor: color,
      loading: this.online == OnlineType.Waiting,
    );
  }

  last(List params) {
    this.isClose = false;
    this.lastTime = RelativeTime.fromInt(params[1]);
    this.lastContent = params[2];
    this.lastReaded = params[3];
  }

  update(List params) {
    if (params[1].length > 2) {
      this.addr = params[1];
    }
    this.name = params[2];
    this.isTop = params[3];
  }

  Session.fromList(List params)
      : this.id = params[0],
        this.fid = params[1],
        this.pid = params[2],
        this.addr = params[3],
        this.type = SessionTypeExtension.fromInt(params[4]),
        this.name = params[5],
        this.isTop = params[6],
        this.isClose = params[7],
        this.lastTime = RelativeTime.fromInt(params[8]),
        this.lastContent = params[9],
        this.lastReaded = params[10],
        this.online = OnlineType.Lost;
}
