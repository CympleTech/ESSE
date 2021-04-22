import 'dart:async';
import "dart:collection";
import 'dart:typed_data';

import 'package:flutter/material.dart';

import 'package:esse/account.dart';
import 'package:esse/utils/logined_cache.dart';
import 'package:esse/widgets/default_core_show.dart';
import 'package:esse/global.dart';
import 'package:esse/rpc.dart';

const DEFAULT_ONLINE_INIT = 8;
const DEFAULT_ONLINE_DELAY = 5;

class AccountProvider extends ChangeNotifier {
  Map<String, Account> accounts = {}; // account's gid and account.
  String activedAccountId; // actived account gid.
  Account get activedAccount => this.accounts[activedAccountId];

  Set<int> topKeys = Set();

  /// current user's did.
  String get id => this.activedAccount.id;

  Widget coreShowWidget = DefaultCoreShow();
  bool systemAppFriendAddNew = false;
  bool systemAppGroupAddNew = false;

  AccountProvider() {
    // rpc notice when account not actived.
    rpc.addNotice(_accountNotice);

    // rpc
    rpc.addListener('account-system-info', _systemInfo, false);
    rpc.addListener('account-update', _accountUpdate, false);
    rpc.addListener('account-login', _accountLogin, false);

    systemInfo();
  }

  handleTops() {
    //
  }

  /// when security load accounts from rpc.
  initAccounts(Map accounts) {
    this.accounts = accounts;
    initLogined(this.accounts.values.toList());
  }

  /// when security load accounts from cache.
  autoAccounts(String gid, Map accounts) {
    Global.changeGid(gid);
    this.activedAccountId = gid;
    this.accounts = accounts;

    this.activedAccount.online = true;
    rpc.send('account-login', [gid, this.activedAccount.lock]);
    new Future.delayed(Duration(seconds: DEFAULT_ONLINE_INIT),
      () => rpc.send('account-online', [gid]));

    this.accounts.forEach((k, v) {
      if (k != gid && v.online) {
        rpc.send('account-login', [v.gid, v.lock]);
        new Future.delayed(Duration(seconds: DEFAULT_ONLINE_INIT),
            () => rpc.send('account-online', [v.gid]));
      }
    });
  }

  /// when security add account.
  addAccount(Account account) {
    Global.changeGid(account.gid);
    this.activedAccountId = account.gid;
    this.accounts[account.gid] = account;

    rpc.send('account-login', [account.gid, account.lock]);
    new Future.delayed(Duration(seconds: DEFAULT_ONLINE_DELAY),
        () => rpc.send('account-online', [account.gid]));
    updateLogined(account);
  }

  updateActivedAccount(String gid) {
    Global.changeGid(gid);
    this.clearActivedAccount();
    this.activedAccountId = gid;
    this.activedAccount.hasNew = false;

    rpc.send('friend-list', []);

    if (!this.activedAccount.online) {
      this.activedAccount.online = true;
      rpc.send('account-login', [gid, this.activedAccount.lock]);
      new Future.delayed(Duration(seconds: DEFAULT_ONLINE_DELAY),
          () => rpc.send('account-online', [gid]));
    }

    mainLogined(gid);
    notifyListeners();
  }

  logout() {
    this.accounts.clear();
    this.clearActivedAccount();
    rpc.send('account-logout', []);
    clearLogined();
  }

  onlineAccount(String gid, String lock) {
    this.accounts[gid].online = true;
    updateLogined(this.accounts[gid]);

    rpc.send('account-login', [gid, lock]);

    new Future.delayed(Duration(seconds: DEFAULT_ONLINE_DELAY),
        () => rpc.send('account-online', [gid]));
    notifyListeners();
  }

  offlineAccount(String gid) {
    this.accounts[gid].online = false;
    updateLogined(this.accounts[gid]);

    if (gid == this.activedAccountId) {
      this.clearActivedAccount();
    }
    rpc.send('account-offline', [gid]);

    notifyListeners();
  }

  updateActivedApp(Widget widget) {
    this.coreShowWidget = widget;
    this.systemAppFriendAddNew = false;
    notifyListeners();
  }

  clearActivedAccount() {
    this.topKeys.clear();
  }

  accountUpdate(String name, [Uint8List avatar]) {
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

  accountPin(String lock) {
    this.activedAccount.lock = lock;
    updateLogined(this.activedAccount);
    notifyListeners();
  }

  systemInfo() {
    rpc.send('account-system-info', []);
  }

  // -- callback when receive rpc info. -- //
  _systemInfo(List params) {
    Global.addr = '0x' + params[0];
  }

  _accountLogin(List _params) {
    // nothing.
  }

  _accountNotice(String gid) {
    if (this.accounts.containsKey(gid)) {
      this.accounts[gid].hasNew = true;
      notifyListeners();
    }
  }

  _accountUpdate(List params) {
    final gid = params[0];
    this.accounts[gid].name = params[1];
    if (params[2].length > 1) {
      this.accounts[gid].updateAvatar(params[2]);
    }
    notifyListeners();
  }
}
