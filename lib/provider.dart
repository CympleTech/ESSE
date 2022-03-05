import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/material.dart';

import 'package:esse/account.dart';
import 'package:esse/utils/logined_cache.dart';
import 'package:esse/widgets/default_core_show.dart';
import 'package:esse/global.dart';
import 'package:esse/rpc.dart';
import 'package:esse/session.dart';

const DEFAULT_ONLINE_INIT = 8;
const DEFAULT_ONLINE_DELAY = 5;

class AccountProvider extends ChangeNotifier {
  Map<String, Account> accounts = {}; // account's pid and account.
  String activedAccountId = ''; // actived account pid.
  Account get activedAccount => this.accounts[activedAccountId]!;

  /// current user's did.
  String get id => this.activedAccount.pid;

  bool systemAppFriendAddNew = false;


  /// home sessions. sorded by last_time.
  Map<int, Session> sessions = {};
  List<int> topKeys = [];
  List<int> orderKeys = [];

  /// actived session.
  int actived = 0;
  Session get activedSession => this.sessions[actived]!;

  /// right main screen show session details.
  Widget coreShowWidget = DefaultCoreShow();

  void orderSessions(int id) {
    if (this.orderKeys.length == 0 || this.orderKeys[0] != id) {
      this.orderKeys.remove(id);
      this.orderKeys.insert(0, id);
    }
  }

  AccountProvider() {
    // rpc notice when account not actived.
    rpc.addNotice(_accountNotice);

    // rpc
    rpc.addListener('account-update', _accountUpdate);
    rpc.addListener('account-login', _accountLogin);

    rpc.addListener('session-list', _sessionList);
    rpc.addListener('session-last', _sessionLast, true);
    rpc.addListener('session-create', _sessionCreate, true);
    rpc.addListener('session-update', _sessionUpdate);
    rpc.addListener('session-close', _sessionClose);
    rpc.addListener('session-delete', _sessionDelete);
    rpc.addListener('session-connect', _sessionConnect);
    rpc.addListener('session-suspend', _sessionSuspend);
    rpc.addListener('session-lost', _sessionLost);
    rpc.addListener('notice-menu', _noticeMenu, true);
  }

  /// when security load accounts from cache.
  autoAccounts(String pid, String pin, Map<String, Account> accounts) {
    Global.changePid(pid);
    this.accounts = accounts;

    this.activedAccountId = pid;
    this.activedAccount.online = true;
    this.activedAccount.pin = pin;

    rpc.send('session-list', []);

    initLogined(pid, this.accounts.values.toList());
    this.coreShowWidget = DefaultCoreShow();
  }

  /// when security add account.
  addAccount(Account account, String pin) {
    Global.changePid(account.pid);
    this.accounts[account.pid] = account;

    this.activedAccountId = account.pid;
    this.activedAccount.online = true;
    this.activedAccount.pin = pin;

    rpc.send('session-list', []);
    updateLogined(account);
  }

  updateActivedAccount(String pid, String pin) {
    Global.changePid(pid);
    this.clearActivedAccount();

    this.activedAccountId = pid;
    this.activedAccount.online = true;
    this.activedAccount.pin = pin;
    this.activedAccount.hasNew = false;

    this.coreShowWidget = DefaultCoreShow();

    // load sessions.
    this.actived = 0;
    this.sessions.clear();
    this.orderKeys.clear();
    rpc.send('session-list', []);

    if (!this.activedAccount.online) {
      this.activedAccount.online = true;
    }

    mainLogined(pid);
    notifyListeners();
  }

  logout() {
    this.actived = 0;
    this.accounts.clear();
    this.clearActivedAccount();
    this.sessions.clear();
    this.orderKeys.clear();
    this.topKeys.clear();

    rpc.send('account-logout', []);
    clearLogined();
  }

  onlineAccount(String pid, String pin) {
    this.accounts[pid]!.online = true;
    this.accounts[pid]!.pin = pin;

    rpc.send('account-login', [pid, pin]);
    notifyListeners();
  }

  offlineAccount(String pid) {
    this.accounts[pid]!.online = false;
    this.accounts[pid]!.pin = '';

    if (pid == this.activedAccountId) {
      this.clearActivedAccount();
    }
    rpc.send('account-offline', [pid]);

    notifyListeners();
  }

  clearActivedAccount() {
    this.topKeys.clear();
  }

  accountUpdate(String name, [Uint8List? avatar]) {
    this.activedAccount.name = name;

    if (avatar != null && avatar.length > 0) {
      this.activedAccount.avatar = avatar;
      rpc.send('account-update', [name, this.activedAccount.encodeAvatar()]);
    } else {
      rpc.send('account-update', [name, '']);
    }
    updateLogined(this.activedAccount);

    notifyListeners();
  }

  accountPin(String pin) {
    this.activedAccount.pin = pin;
    notifyListeners();
  }

  clearActivedSession(SessionType type) {
    if (this.actived > 0 && this.activedSession.type == type) {
      rpc.send('session-suspend', [this.actived, this.activedSession.pid,
          this.activedSession.type == SessionType.Group]
      );
      this.actived = 0;
      this.coreShowWidget = DefaultCoreShow();
    }
  }

  updateActivedSession(int id, [SessionType type = SessionType.Chat, int fid = 0]) {
    if (fid > 0) {
      for (int k in this.sessions.keys) {
        final v = this.sessions[k]!;
        if (v.type == type && v.fid == fid) {
          id = k;
          break;
        }
      }
    }

    if (id > 0) {
      if (this.actived != id && this.actived > 0) {
        rpc.send('session-suspend', [this.actived, this.activedSession.pid,
            this.activedSession.type == SessionType.Group]
        );
      }
      this.actived = id;
      this.activedSession.lastReaded = true;
      final online = this.activedSession.online;
      if (online == OnlineType.Lost || online == OnlineType.Suspend) {
        if (online == OnlineType.Lost) {
          this.activedSession.online = OnlineType.Waiting;
          Timer(Duration(seconds: 10), () {
              if (this.sessions[id] != null && this.sessions[id]!.online == OnlineType.Waiting) {
                this.sessions[id]!.online = OnlineType.Lost;
                notifyListeners();
              }
          });
        }
        rpc.send('session-connect', [id, this.activedSession.pid]);
        notifyListeners();
      }
    }
  }

  updateActivedWidget(Widget? coreWidget) {
    if (coreWidget != null) {
      print("update actived widget");
      this.coreShowWidget = coreWidget;
    } else {
      this.actived = 0;
      this.coreShowWidget = DefaultCoreShow();
    }
    notifyListeners();
  }

  // -- callback when receive rpc info. -- //
  _accountLogin(List _params) {
    // nothing.
  }

  _accountNotice(String pid) {
    if (this.accounts.containsKey(pid)) {
      this.accounts[pid]!.hasNew = true;
      notifyListeners();
    }
  }

  _noticeMenu(List params) {
    final st = SessionTypeExtension.fromInt(params[0]);
    if (st == SessionType.Chat) {
      this.systemAppFriendAddNew = true;
      notifyListeners();
    }
  }

  _accountUpdate(List params) {
    final pid = params[0];
    this.accounts[pid]!.name = params[1];
    if (params[2].length > 1) {
      this.accounts[pid]!.updateAvatar(params[2]);
    }
    notifyListeners();
  }

  _sessionList(List params) {
    this.sessions.clear();
    this.orderKeys.clear();
    this.topKeys.clear();

    params.forEach((params) {
        final id = params[0];
        this.sessions[id] = Session.fromList(params);
        if (!this.sessions[id]!.isClose) {
          if (this.sessions[id]!.isTop) {
            this.topKeys.add(id);
          } else {
            this.orderKeys.add(id);
          }
        }
    });
    notifyListeners();
  }

  _sessionCreate(List params) {
    final id = params[0];
    this.sessions[id] = Session.fromList(params);
    orderSessions(id);
    notifyListeners();
  }

  _sessionLast(List params) {
    final id = params[0];
    this.sessions[id]!.last(params);
    if (id == this.actived && !this.sessions[id]!.lastReaded) {
      rpc.send('session-readed', [id]);
      this.sessions[id]!.lastReaded = true;
    }
    orderSessions(id);
    notifyListeners();
  }

  _sessionUpdate(List params) {
    final id = params[0];
    this.sessions[id]!.update(params);
    if (this.sessions[id]!.isTop) {
      this.topKeys.add(id);
      this.orderKeys.remove(id);
    } else {
      orderSessions(id);
      this.topKeys.remove(id);
    }
    notifyListeners();
  }

  _sessionClose(List params) {
    final id = params[0];
    this.sessions[id]!.isClose = true;
    notifyListeners();
  }

  _sessionDelete(List params) {
    final id = params[0];
    this.sessions.remove(id);
    this.orderKeys.remove(id);
    this.topKeys.remove(id);
    if (id == this.actived) {
      this.actived = 0;
      this.coreShowWidget = DefaultCoreShow();
    }
    notifyListeners();
  }

  _sessionConnect(List params) {
    final id = params[0];
    final addr = params[1];
    this.sessions[id]!.addr = addr;
    this.sessions[id]!.online = OnlineType.Active;
    notifyListeners();
  }

  _sessionSuspend(List params) {
    final id = params[0];
    this.sessions[id]!.online = OnlineType.Suspend;
    notifyListeners();
  }

  _sessionLost(List params) {
    final id = params[0];
    this.sessions[id]!.online = OnlineType.Lost;
    notifyListeners();
  }
}
