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
  Files,
  Device,
  Assistant,
  Domain,
  Service,
}

extension SessionTypeExtension on SessionType {
  static SessionType fromInt(int s) {
    switch (s) {
      case 0:
        return SessionType.Chat;
      case 1:
        return SessionType.Group;
      case 2:
        return SessionType.Files;
      case 3:
        return SessionType.Device;
      case 4:
        return SessionType.Assistant;
      case 5:
        return SessionType.Domain;
      case 6:
        return SessionType.Service;
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
  String gid;
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
        borderRadius: BorderRadius.circular(15.0),
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

  bool isActive()  {
    return this.online == OnlineType.Active || this.online == OnlineType.Suspend;
  }

  List parse(AppLocalizations lang) {
    switch (this.type) {
      case SessionType.Chat:
        return [showAvatar(), this.name, this.lastContent, this.lastTime.toString()];
      case SessionType.Assistant:
        final params = Session.innerService(InnerService.Assistant, lang);
        return [params[0], params[1], params[2], ''];
      case SessionType.Files:
        final params = Session.innerService(InnerService.Files, lang);
        return [params[0], params[1], params[2], ''];
    }
  }

  Avatar showAvatar({double width = 45.0}) {
    final avatar = Global.avatarPath + this.gid + '.png';

    Color color;
    switch (this.online) {
      case OnlineType.Active:
        color = Color(0xFF0EE50A);
        break;
      case OnlineType.Suspend:
        color = Colors.blue;
        break;
    }

    return Avatar(
      width: width,
      name: this.name,
      avatarPath: avatar,
      online: this.online != OnlineType.Lost,
      onlineColor: color,
      hasNew: !this.lastReaded,
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
    this.addr = params[1];
    this.name = params[2];
    this.isTop = params[3];
  }

  Session.fromList(List params) {
    this.id = params[0];
    this.fid = params[1];
    this.gid = params[2];
    this.addr = params[3];
    this.type = SessionTypeExtension.fromInt(params[4]);
    this.name = params[5];
    this.isTop = params[6];
    this.isClose = params[7];
    this.lastTime = RelativeTime.fromInt(params[8]);
    this.lastContent = params[9];
    this.lastReaded = params[10];
    this.online = OnlineType.Lost;
  }
}
